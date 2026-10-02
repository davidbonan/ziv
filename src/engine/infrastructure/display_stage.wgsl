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
    // source region to show, in texture coordinates: min.xy, size.zw
    region: vec4f,
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
    adjustments: Adjustments,
    masks: array<Mask, 16>,
    // the mask whose coverage the overlay shows
    overlaid: Mask,
}

@group(0) @binding(0) var<uniform> stage: DisplayStage;
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(2) var source_sampler: sampler;
@group(0) @binding(3) var coverage_layers: texture_2d_array<f32>;
@group(0) @binding(4) var coverage_sampler: sampler;
@group(0) @binding(5) var enhancement: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
}

@vertex
fn vertex(@builtin(vertex_index) index: u32) -> VertexOutput {
    let uv = vec2f(f32((index << 1u) & 2u), f32(index & 2u));
    var output: VertexOutput;
    output.position = vec4f(uv * vec2f(2.0, -2.0) + vec2f(-1.0, 1.0), 0.0, 1.0);
    output.uv = stage.region.xy + uv * stage.region.zw;
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
        return textureSampleLevel(coverage_layers, coverage_sampler, point / stage.photo.xy, layer, 0.0).r;
    }
    return linear_gradient_coverage(mask.geometry, point);
}

fn coverage(mask: Mask, point: vec2f) -> f32 {
    let covered = shape_coverage(mask, point);
    return mix(covered, 1.0 - covered, mask.coverage.x);
}

fn develop(working: vec3f, point: vec2f) -> vec3f {
    var colour = adjusted(working, stage.adjustments);
    let mask_count = u32(stage.photo.z);
    for (var mask_index = 0u; mask_index < mask_count; mask_index++) {
        let locally = adjusted(colour, stage.masks[mask_index].adjustments);
        colour = mix(colour, locally, coverage(stage.masks[mask_index], point));
    }
    return colour;
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

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4f {
    let point = input.uv * stage.photo.xy;
    let working = develop(enhanced(input.uv), point);
    let linear = vec3f(
        dot(stage.working_to_display_red.xyz, working),
        dot(stage.working_to_display_green.xyz, working),
        dot(stage.working_to_display_blue.xyz, working),
    );
    let display = encode(clamp(base_rendered(linear), vec3f(0.0), vec3f(1.0)));
    let veil = coverage(stage.overlaid, point) * stage.overlay.a;
    return vec4f(mix(display, stage.overlay.rgb, veil), 1.0);
}
