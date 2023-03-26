use crate::prelude::*;

pub fn recursive_layout(
    view_map: &ViewMap,
    vk: ViewKey,
    rect: Rect,
    layout_rects: &mut HashMap<ViewKey, Rect>,
) {
    // Stash the layout rects as we go.
    assert!(!layout_rects.contains_key(&vk));
    // rect comes in in absolute coordinates.
    layout_rects.insert(vk, rect);

    // Allow the view to layout itself and its children.
    let next_jobs = view_map.get_view(vk).layout(&view_map, rect.size());

    // Recurse by running layout on the child nodes.
    next_jobs.into_iter().for_each(|(vk, child_rect)| {
        recursive_layout(view_map, vk, child_rect + rect.top_left(), layout_rects);
    });
}
