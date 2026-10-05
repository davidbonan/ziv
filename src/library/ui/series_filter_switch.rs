use crate::library::domain::mark::MOST_STARS;
use crate::library::domain::series_filter::{SeriesFilter, stars_or_more};

use super::mark_line::star_button;

pub const ALL_PHOTOS_LABEL: &str = "All";

/// "All", then five stars lit up to the filter. Returns the filter the user asked for.
pub fn series_filter_switch(ui: &mut egui::Ui, filter: SeriesFilter) -> Option<SeriesFilter> {
    ui.spacing_mut().item_spacing.x = 0.0;
    let all = egui::Button::new(ALL_PHOTOS_LABEL)
        .frame_when_inactive(false)
        .selected(filter == SeriesFilter::All);
    let all_asked = ui.add(all).clicked().then_some(SeriesFilter::All);
    let stars_asked = (1..=MOST_STARS)
        .filter(|stars| star_button(ui, stars_or_more(*stars), *stars <= filter.stars()).clicked())
        .last();
    stars_asked.map(SeriesFilter::at_least).or(all_asked)
}
