struct Adjustments {
    white_balance_red: vec4f,
    white_balance_green: vec4f,
    white_balance_blue: vec4f,
    // exposure gain, contrast slope, highlights shift, shadows shift (stops)
    exposure_and_tone: vec4f,
    // whites stretch, blacks offset, vibrance, saturation factor
    whites_blacks_and_presence: vec4f,
}

// The kinds of shape, as `display_stage.rs` numbers them.
const LINEAR_GRADIENT: f32 = 0.0;
const RADIAL_GRADIENT: f32 = 1.0;
const RECTANGLE: f32 = 2.0;
const POLYGON: f32 = 3.0;
const COVERAGE_LAYER: f32 = 4.0;

struct Mask {
    adjustments: Adjustments,
    // 1 when inverted else 0, kind of shape, feather 0…1,
    // number of corners of a polygon or layer of a mask rendered from an image
    coverage: vec4f,
    // linear gradient: full coverage point .xy, no coverage point .zw (photo units)
    // radial gradient: centre .xy, radii .zw
    // rectangle: centre .xy, half size .zw
    geometry: vec4f,
    // radial gradient: cosine and sine of the rotation .xy
    orientation: vec4f,
    // polygon: two corners a row, .xy then .zw
    corners: array<vec4f, 8>,
}

struct DisplayStage {
    working_to_display_red: vec4f,
    working_to_display_green: vec4f,
    working_to_display_blue: vec4f,
    // linear slope, linear cutoff, offset, gamma
    transfer: vec4f,
    // source region to show, in texture coordinates: its top-left corner .xy,
    region_origin: vec4f,
    // what its top side .xy and its left side .zw span
    region_sides: vec4f,
    // middle grey, stops from middle grey to white, highlights width, shadows width (stops)
    tone_shape: vec4f,
    // luminance where blacks stops acting, darkest luminance handled
    tone_limits: vec4f,
    luminance: vec4f,
    // tone curve contrast, shoulder, 1 when the curve applies else 0
    base_rendering: vec4f,
    // bottom-right corner of the photo in photo units .xy, number of masks .z,
    // share of the enhancement in the photo .w
    photo: vec4f,
    // veil colour .rgb (encoded), opacity at full coverage .a: 0 without overlay
    overlay: vec4f,
    // shortest gradient squared, smallest radius or feather, reach of a full feather
    mask_limits: vec4f,
    // 1 when the tone curves apply else 0
    tone_curves: vec4f,
    // chroma under which a color fades out of its range, 1 when the color mixer applies else 0
    color_mixer: vec4f,
    // per color range, in the order of the hues:
    // centre (radians), hue turn (radians), chroma scale, lightness scale
    color_ranges: array<vec4f, 8>,
    working_to_lms: array<vec4f, 3>,
    lms_to_lab: array<vec4f, 3>,
    lab_to_lms: array<vec4f, 3>,
    lms_to_working: array<vec4f, 3>,
    // display luma weights .rgb, 1 when color grading applies else 0
    grading_luma: vec4f,
    // balance exponent, half the passage between two zones, shadows edge, highlights edge
    grading_zones: vec4f,
    // what a tone fully in a zone is moved by: shadows, midtones, highlights, global
    grading_offsets: array<vec4f, 4>,
    adjustments: Adjustments,
    masks: array<Mask, 16>,
    // the mask whose coverage the overlay shows
    overlaid: Mask,
}

@group(0) @binding(0) var<uniform> stage: DisplayStage;
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(2) var source_sampler: sampler;
@group(0) @binding(3) var coverage_layers: texture_2d_array<f32>;
@group(0) @binding(4) var linear_sampler: sampler;
@group(0) @binding(5) var enhancement: texture_2d<f32>;
@group(0) @binding(6) var tone_curve_lookup: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
}

@vertex
fn vertex(@builtin(vertex_index) index: u32) -> VertexOutput {
    let uv = vec2f(f32((index << 1u) & 2u), f32(index & 2u));
    var output: VertexOutput;
    output.position = vec4f(uv * vec2f(2.0, -2.0) + vec2f(-1.0, 1.0), 0.0, 1.0);
    output.uv = stage.region_origin.xy + uv.x * stage.region_sides.xy + uv.y * stage.region_sides.zw;
    return output;
}

fn encode(linear: vec3f) -> vec3f {
    let slope = stage.transfer.x;
    let cutoff = stage.transfer.y;
    let offset = stage.transfer.z;
    let gamma = stage.transfer.w;
    let low = linear * slope;
    let high = (1.0 + offset) * pow(linear, vec3f(1.0 / gamma)) - offset;
    return select(high, low, linear <= vec3f(cutoff));
}

fn smooth_step(position: f32) -> f32 {
    let clamped = clamp(position, 0.0, 1.0);
    return clamped * clamped * (3.0 - 2.0 * clamped);
}

fn contrasted(stops: f32, slope: f32) -> f32 {
    let white = stage.tone_shape.y;
    if stops <= 0.0 {
        return slope * stops;
    }
    if stops < white {
        return slope * stops + (1.0 - slope) / white * stops * stops;
    }
    return white + (2.0 - slope) * (stops - white);
}

fn with_highlights_and_shadows(stops: f32, shifts: vec2f) -> f32 {
    let highlights = shifts.x * smooth_step(stops / stage.tone_shape.z);
    let shadows = shifts.y * smooth_step(-stops / stage.tone_shape.w);
    return stops + highlights + shadows;
}

fn with_whites(stops: f32, stretch: f32) -> f32 {
    let above_grey = max(stops, 0.0);
    return stops + stretch * above_grey * above_grey / (above_grey + 1.0);
}

fn black_lift(luminance: f32, offset: f32) -> f32 {
    let nearness_to_black = max(1.0 - luminance / stage.tone_limits.x, 0.0);
    return -offset * nearness_to_black * nearness_to_black;
}

fn colorfulness(working: vec3f) -> f32 {
    let strongest = max(working.r, max(working.g, working.b));
    let weakest = min(working.r, min(working.g, working.b));
    if strongest <= 0.0 {
        return 0.0;
    }
    return clamp((strongest - weakest) / strongest, 0.0, 1.0);
}

fn with_presence(working: vec3f, vibrance_and_saturation: vec2f) -> vec3f {
    let grey = dot(stage.luminance.xyz, working);
    let vibrance = 1.0 + vibrance_and_saturation.x * (1.0 - colorfulness(working));
    let scale = vibrance * vibrance_and_saturation.y;
    return vec3f(grey) + (working - vec3f(grey)) * scale;
}

fn adjusted(working: vec3f, adjustments: Adjustments) -> vec3f {
    let middle_grey = stage.tone_shape.x;
    let tone = adjustments.exposure_and_tone;
    let balanced = vec3f(
        dot(adjustments.white_balance_red.xyz, working),
        dot(adjustments.white_balance_green.xyz, working),
        dot(adjustments.white_balance_blue.xyz, working),
    );
    let exposed = balanced * tone.x;
    let luminance = max(dot(stage.luminance.xyz, exposed), stage.tone_limits.y);
    let stops = log2(luminance / middle_grey);
    let whites = adjustments.whites_blacks_and_presence.x;
    let toned = with_whites(with_highlights_and_shadows(contrasted(stops, tone.y), tone.zw), whites);
    let scaled = middle_grey * exp2(toned);
    let lift = black_lift(scaled, adjustments.whites_blacks_and_presence.y);
    return with_presence(
        exposed * (scaled / luminance) + vec3f(lift),
        adjustments.whites_blacks_and_presence.zw,
    );
}

fn linear_gradient_coverage(geometry: vec4f, point: vec2f) -> f32 {
    let along = geometry.zw - geometry.xy;
    let length_squared = max(dot(along, along), stage.mask_limits.x);
    return 1.0 - smooth_step(dot(point - geometry.xy, along) / length_squared);
}

fn radial_gradient_coverage(mask: Mask, point: vec2f) -> f32 {
    let smallest = stage.mask_limits.y;
    let from_centre = point - mask.geometry.xy;
    let turn = mask.orientation.xy;
    let along = vec2f(
        dot(from_centre, vec2f(turn.x, turn.y)),
        dot(from_centre, vec2f(-turn.y, turn.x)),
    );
    let distance = length(along / max(mask.geometry.zw, vec2f(smallest)));
    let feather = max(mask.coverage.z, smallest);
    return 1.0 - smooth_step((distance - (1.0 - feather)) / feather);
}

// Coverage at `depth` inside an outline: none on it, full once the feather is crossed.
fn feathered_coverage(depth: f32, feather: f32) -> f32 {
    let width = max(feather * stage.mask_limits.z, stage.mask_limits.y);
    return smooth_step(depth / width);
}

fn rectangle_depth(geometry: vec4f, point: vec2f) -> f32 {
    let depths = geometry.zw - abs(point - geometry.xy);
    return min(depths.x, depths.y);
}

fn polygon_depth(mask: Mask, point: vec2f) -> f32 {
    var corners = mask.corners;
    let count = u32(mask.coverage.w);
    var nearest_squared = 3.0e38;
    var inside = false;
    let last = corners[(count - 1u) / 2u];
    var previous = select(last.zw, last.xy, (count - 1u) % 2u == 0u);
    for (var index = 0u; index < count; index++) {
        let pair = corners[index / 2u];
        let corner = select(pair.zw, pair.xy, index % 2u == 0u);
        let edge = previous - corner;
        let to_point = point - corner;
        let along = clamp(dot(to_point, edge) / max(dot(edge, edge), stage.mask_limits.x), 0.0, 1.0);
        let to_edge = to_point - edge * along;
        nearest_squared = min(nearest_squared, dot(to_edge, to_edge));
        let is_below_corner = point.y >= corner.y;
        let is_above_previous = point.y < previous.y;
        let is_right_of_edge = edge.x * to_point.y > edge.y * to_point.x;
        let crossing = vec3<bool>(is_below_corner, is_above_previous, is_right_of_edge);
        if all(crossing) || !any(crossing) {
            inside = !inside;
        }
        previous = corner;
    }
    return select(-sqrt(nearest_squared), sqrt(nearest_squared), inside);
}

fn shape_coverage(mask: Mask, point: vec2f) -> f32 {
    let kind = mask.coverage.y;
    if kind == RADIAL_GRADIENT {
        return radial_gradient_coverage(mask, point);
    }
    if kind == RECTANGLE {
        return feathered_coverage(rectangle_depth(mask.geometry, point), mask.coverage.z);
    }
    if kind == POLYGON {
        return feathered_coverage(polygon_depth(mask, point), mask.coverage.z);
    }
    if kind == COVERAGE_LAYER {
        let layer = i32(mask.coverage.w);
        return textureSampleLevel(coverage_layers, linear_sampler, point / stage.photo.xy, layer, 0.0).r;
    }
    return linear_gradient_coverage(mask.geometry, point);
}

fn coverage(mask: Mask, point: vec2f) -> f32 {
    let covered = shape_coverage(mask, point);
    return mix(covered, 1.0 - covered, mask.coverage.x);
}

const TURN: f32 = 6.283185307179586;
const RANGE_COUNT: u32 = 8u;

fn through(matrix: array<vec4f, 3>, colour: vec3f) -> vec3f {
    return vec3f(
        dot(matrix[0].xyz, colour),
        dot(matrix[1].xyz, colour),
        dot(matrix[2].xyz, colour),
    );
}

fn cube_root(lms: vec3f) -> vec3f {
    return sign(lms) * pow(abs(lms), vec3f(1.0 / 3.0));
}

// What a color of this hue gets: a share of the two ranges it sits between.
fn range_factors_at(hue_from_axis: f32) -> vec4f {
    let first = stage.color_ranges[0];
    let hue = select(hue_from_axis, hue_from_axis + TURN, hue_from_axis < first.x);
    var before = stage.color_ranges[RANGE_COUNT - 1u];
    var next = vec4f(first.x + TURN, first.yzw);
    for (var index = 1u; index < RANGE_COUNT; index++) {
        if hue < stage.color_ranges[index].x {
            before = stage.color_ranges[index - 1u];
            next = stage.color_ranges[index];
            break;
        }
    }
    let share = smooth_step((hue - before.x) / (next.x - before.x));
    return vec4f(hue, mix(before.yzw, next.yzw, share));
}

fn color_mixed(working: vec3f) -> vec3f {
    let lab = through(stage.lms_to_lab, cube_root(through(stage.working_to_lms, working)));
    let chroma = length(lab.yz);
    if stage.color_mixer.y == 0.0 || chroma <= 0.0 {
        return working;
    }
    // hue, hue turn, chroma scale, lightness scale
    let factors = range_factors_at(atan2(lab.z, lab.y));
    let fade = smooth_step(chroma / stage.color_mixer.x);
    let turned = factors.x + factors.y * fade;
    let scaled = chroma * factors.z;
    let lightened = 1.0 + (factors.w - 1.0) * fade;
    let mixed = vec3f(lab.x, scaled * cos(turned), scaled * sin(turned)) * lightened;
    let lms = through(stage.lab_to_lms, mixed);
    return through(stage.lms_to_working, lms * lms * lms);
}

fn develop(working: vec3f, point: vec2f) -> vec3f {
    var colour = adjusted(working, stage.adjustments);
    let mask_count = u32(stage.photo.z);
    for (var mask_index = 0u; mask_index < mask_count; mask_index++) {
        let locally = adjusted(colour, stage.masks[mask_index].adjustments);
        colour = mix(colour, locally, coverage(stage.masks[mask_index], point));
    }
    return color_mixed(colour);
}

fn base_rendered(linear: vec3f) -> vec3f {
    let contrast = stage.base_rendering.x;
    let shoulder = stage.base_rendering.y;
    let steepened = pow(max(linear, vec3f(0.0)), vec3f(contrast));
    let curved = steepened * (1.0 + shoulder) / (steepened + vec3f(shoulder));
    return mix(linear, curved, stage.base_rendering.z);
}

fn enhanced(uv: vec2f) -> vec3f {
    let original = textureSample(source, source_sampler, uv).rgb;
    let enhancement = textureSample(enhancement, source_sampler, uv).rgb;
    return mix(original, enhancement, stage.photo.w);
}

// How much of a tone is past the passage centred on `edge`.
fn past_zone_edge(edge: f32, tone: f32) -> f32 {
    let passage = stage.grading_zones.y;
    return smooth_step((tone - (edge - passage)) / (2.0 * passage));
}

fn color_graded(encoded: vec3f) -> vec3f {
    let luma = clamp(dot(stage.grading_luma.rgb, encoded), 0.0, 1.0);
    let tone = pow(luma, stage.grading_zones.x);
    let past_shadows = past_zone_edge(stage.grading_zones.z, tone);
    let in_highlights = past_zone_edge(stage.grading_zones.w, tone);
    let moved = (1.0 - past_shadows) * stage.grading_offsets[0].rgb
        + (past_shadows - in_highlights) * stage.grading_offsets[1].rgb
        + in_highlights * stage.grading_offsets[2].rgb
        + stage.grading_offsets[3].rgb;
    let graded = clamp(encoded + moved, vec3f(0.0), vec3f(1.0));
    return mix(encoded, graded, stage.grading_luma.w);
}

fn tone_curved(encoded: vec3f) -> vec3f {
    let size = f32(textureDimensions(tone_curve_lookup).x);
    let at = (encoded * (size - 1.0) + vec3f(0.5)) / size;
    let curved = vec3f(
        textureSampleLevel(tone_curve_lookup, linear_sampler, vec2f(at.r, 0.5), 0.0).r,
        textureSampleLevel(tone_curve_lookup, linear_sampler, vec2f(at.g, 0.5), 0.0).g,
        textureSampleLevel(tone_curve_lookup, linear_sampler, vec2f(at.b, 0.5), 0.0).b,
    );
    return mix(encoded, curved, stage.tone_curves.x);
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4f {
    let point = input.uv * stage.photo.xy;
    let working = develop(enhanced(input.uv), point);
    let linear = vec3f(
        dot(stage.working_to_display_red.xyz, working),
        dot(stage.working_to_display_green.xyz, working),
        dot(stage.working_to_display_blue.xyz, working),
    );
    let encoded = encode(clamp(base_rendered(linear), vec3f(0.0), vec3f(1.0)));
    let display = tone_curved(color_graded(encoded));
    let veil = coverage(stage.overlaid, point) * stage.overlay.a;
    // Clamped sampling smears the edge of the source: past a texel, nothing is shown.
    let texel = 1.0 / vec2f(textureDimensions(source));
    let is_outside = any(input.uv < -texel) || any(input.uv > vec2f(1.0) + texel);
    return select(vec4f(mix(display, stage.overlay.rgb, veil), 1.0), vec4f(0.0), is_outside);
}
