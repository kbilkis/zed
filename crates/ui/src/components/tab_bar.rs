use std::rc::Rc;

use gpui::{AnyElement, Bounds, Pixels, ScrollHandle, canvas};
use smallvec::SmallVec;

use crate::Tab;
use crate::prelude::*;

#[derive(IntoElement, RegisterComponent)]
pub struct TabBar {
    id: ElementId,
    start_children: SmallVec<[AnyElement; 2]>,
    children: SmallVec<[AnyElement; 2]>,
    end_children: SmallVec<[AnyElement; 2]>,
    scroll_handle: Option<ScrollHandle>,
    wrap: bool,
    report_bounds: Option<Rc<dyn Fn(Bounds<Pixels>)>>,
    actions_width_reporter: Option<Rc<dyn Fn(Bounds<Pixels>)>>,
}

impl TabBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            start_children: SmallVec::new(),
            children: SmallVec::new(),
            end_children: SmallVec::new(),
            scroll_handle: None,
            wrap: false,
            report_bounds: None,
            actions_width_reporter: None,
        }
    }

    pub fn track_scroll(mut self, scroll_handle: &ScrollHandle) -> Self {
        self.scroll_handle = Some(scroll_handle.clone());
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Reports the bar's laid-out bounds every frame via a zero-footprint
    /// canvas overlay (wrap layout only). Used by callers to learn the bar's
    /// right edge for row-end tab extension widths.
    pub fn report_bounds(mut self, report: Rc<dyn Fn(Bounds<Pixels>)>) -> Self {
        self.report_bounds = Some(report);
        self
    }

    /// Reports the wrap-mode absolute actions container's bounds (wrap layout
    /// only), so the caller can reserve the first row's right edge and keep
    /// tabs from sliding underneath the top-right buttons.
    pub fn report_actions_bounds(mut self, report: Rc<dyn Fn(Bounds<Pixels>)>) -> Self {
        self.actions_width_reporter = Some(report);
        self
    }

    pub fn start_children_mut(&mut self) -> &mut SmallVec<[AnyElement; 2]> {
        &mut self.start_children
    }

    pub fn start_child(mut self, start_child: impl IntoElement) -> Self
    where
        Self: Sized,
    {
        self.start_children_mut()
            .push(start_child.into_element().into_any());
        self
    }

    pub fn start_children(
        mut self,
        start_children: impl IntoIterator<Item = impl IntoElement>,
    ) -> Self
    where
        Self: Sized,
    {
        self.start_children_mut().extend(
            start_children
                .into_iter()
                .map(|child| child.into_any_element()),
        );
        self
    }

    pub fn end_children_mut(&mut self) -> &mut SmallVec<[AnyElement; 2]> {
        &mut self.end_children
    }

    pub fn end_child(mut self, end_child: impl IntoElement) -> Self
    where
        Self: Sized,
    {
        self.end_children_mut()
            .push(end_child.into_element().into_any());
        self
    }

    pub fn end_children(mut self, end_children: impl IntoIterator<Item = impl IntoElement>) -> Self
    where
        Self: Sized,
    {
        self.end_children_mut().extend(
            end_children
                .into_iter()
                .map(|child| child.into_any_element()),
        );
        self
    }
}

impl ParentElement for TabBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl RenderOnce for TabBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let border_color = cx.theme().colors().border;
        let container_height = Tab::container_height(cx);

        if self.wrap {
            // Buttons participate in the wrapping flow: nav buttons first, pane
            // buttons last. Row 1 shares horizontal space with them; rows 2..N span
            // the full width. Bottom borders come from three cooperating sources,
            // mirroring the single-row layout: the absolute overlay behind
            // everything (covers the drop target and any gap), each button
            // container's own border, and each inactive tab's own border. The
            // lines coincide pixel-for-pixel, so nothing doubles.
            return h_flex()
                .id(self.id)
                .group("tab_bar")
                .flex_wrap()
                .relative()
                .flex_none()
                .w_full()
                .bg(cx.theme().colors().tab_bar_background)
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .border_b_1()
                        .border_color(border_color),
                )
                .when_some(self.report_bounds, |this, report| {
                    this.child(
                        canvas(
                            move |bounds: Bounds<Pixels>, _: &mut Window, _: &mut App| {
                                report(bounds)
                            },
                            |_: Bounds<Pixels>, _: (), _: &mut Window, _: &mut App| {},
                        )
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full(),
                    )
                })
                .when(!self.start_children.is_empty(), |this| {
                    this.child(
                        h_flex()
                            .flex_none()
                            .h(container_height)
                            .gap(DynamicSpacing::Base04.rems(cx))
                            .px(DynamicSpacing::Base06.rems(cx))
                            .border_r_1()
                            .border_b_1()
                            .border_color(border_color)
                            .children(self.start_children),
                    )
                })
                .children(self.children)
                .when(!self.end_children.is_empty(), |this| {
                    // Pane buttons anchor at the top-right corner in wrap mode
                    // (matching every non-wrap layout) rather than flowing at
                    // the end of the last row (which would move them whenever
                    // the row count changes). Out of the wrap flow; the layout
                    // reserves the first row's right edge via the reported
                    // width so tabs never slide underneath.
                    this.child(
                        h_flex()
                            .id("wrap_bar_actions")
                            .absolute()
                            .top_0()
                            .right_0()
                            .h(container_height)
                            .gap(DynamicSpacing::Base04.rems(cx))
                            .px(DynamicSpacing::Base06.rems(cx))
                            .border_l_1()
                            .border_b_1()
                            .bg(cx.theme().colors().tab_bar_background)
                            .border_color(border_color)
                            .children(self.end_children)
                            .when_some(self.actions_width_reporter, |this, report| {
                                this.child(
                                    canvas(
                                        move |bounds: Bounds<Pixels>,
                                              _: &mut Window,
                                              _: &mut App| report(bounds),
                                        |_: Bounds<Pixels>, _: (), _: &mut Window, _: &mut App| {},
                                    )
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .size_full(),
                                )
                            }),
                    )
                })
                .into_any_element();
        }

        div()
            .id(self.id)
            .group("tab_bar")
            .flex()
            .flex_none()
            .w_full()
            .h(container_height)
            .bg(cx.theme().colors().tab_bar_background)
            .when(!self.start_children.is_empty(), |this| {
                this.child(
                    h_flex()
                        .flex_none()
                        .gap(DynamicSpacing::Base04.rems(cx))
                        .px(DynamicSpacing::Base06.rems(cx))
                        .border_b_1()
                        .border_r_1()
                        .border_color(border_color)
                        .children(self.start_children),
                )
            })
            .child(
                div()
                    .relative()
                    .flex_1()
                    .h_full()
                    .overflow_x_hidden()
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .size_full()
                            .border_b_1()
                            .border_color(border_color),
                    )
                    .child(
                        h_flex()
                            .id("tabs")
                            .flex_grow_1()
                            .overflow_x_scroll()
                            .when_some(self.scroll_handle, |cx, scroll_handle| {
                                cx.track_scroll(&scroll_handle)
                            })
                            .children(self.children),
                    ),
            )
            .when(!self.end_children.is_empty(), |this| {
                this.child(
                    h_flex()
                        .flex_none()
                        .gap(DynamicSpacing::Base04.rems(cx))
                        .px(DynamicSpacing::Base06.rems(cx))
                        .border_color(border_color)
                        .border_b_1()
                        .border_l_1()
                        .children(self.end_children),
                )
            })
            .into_any_element()
    }
}

impl Component for TabBar {
    fn scope() -> ComponentScope {
        ComponentScope::Navigation
    }

    fn name() -> &'static str {
        "TabBar"
    }

    fn description() -> &'static str {
        "A horizontal bar containing tabs for navigation between different views \
        or sections."
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> AnyElement {
        v_flex()
            .gap_6()
            .children(vec![
                example_group_with_title(
                    "Basic Usage",
                    vec![
                        single_example(
                            "Empty TabBar",
                            TabBar::new("empty_tab_bar").into_any_element(),
                        ),
                        single_example(
                            "With Tabs",
                            TabBar::new("tab_bar_with_tabs")
                                .child(Tab::new("tab1"))
                                .child(Tab::new("tab2"))
                                .child(Tab::new("tab3"))
                                .into_any_element(),
                        ),
                    ],
                ),
                example_group_with_title(
                    "With Start and End Children",
                    vec![single_example(
                        "Full TabBar",
                        TabBar::new("full_tab_bar")
                            .start_child(Button::new("start_button", "Start"))
                            .child(Tab::new("tab1"))
                            .child(Tab::new("tab2"))
                            .child(Tab::new("tab3"))
                            .end_child(Button::new("end_button", "End"))
                            .into_any_element(),
                    )],
                ),
                example_group_with_title(
                    "Wrap",
                    vec![single_example(
                        "Wrapped Tabs",
                        TabBar::new("wrapped_tab_bar")
                            .wrap(true)
                            .child(Tab::new("tab1"))
                            .child(Tab::new("tab2"))
                            .child(Tab::new("tab3"))
                            .child(Tab::new("tab4"))
                            .child(Tab::new("tab5"))
                            .child(Tab::new("tab6"))
                            .into_any_element(),
                    )],
                ),
            ])
            .into_any_element()
    }
}

#[cfg(test)]
mod wrap_grow_tests {
    use gpui::{TestAppContext, div, px, size};
    use crate::prelude::*;

    /// Confirms the filler-strip mechanism: an EMPTY flex item with
    /// `flex_grow_1` (plus an ABSOLUTE child, which never participates in flex
    /// sizing) receives leftover distribution on wrapped lines — unlike items
    /// whose in-flow content is measured text (see the Label repro below).
    /// This is what lets wrap row-ends fill at layout time with zero latency.
    #[gpui::test]
    fn empty_grow_filler_extends_on_wrapped_lines(cx: &mut TestAppContext) {
        let cx = cx.add_empty_window();
        cx.draw(gpui::Point::default(), size(px(150.), px(100.)), |_, _| {
            crate::h_flex()
                .id("bar")
                .flex_wrap()
                .w_full()
                .child(div().id("c0").h(px(28.)).w(px(50.)).debug_selector(|| "c0".into()))
                .child(div().id("c1").h(px(28.)).w(px(50.)).debug_selector(|| "c1".into()))
                .child(
                    div()
                        .id("filler")
                        .h(px(28.))
                        .flex_grow_1()
                        .min_w_0()
                        .relative()
                        // absolute child: must not affect the filler's flex sizing
                        .child(
                            div()
                                .id("filler_child")
                                .absolute()
                                .top_0()
                                .right_0()
                                .w(px(28.))
                                .h(px(28.)),
                        )
                        .debug_selector(|| "filler".into()),
                )
                .child(div().id("c2").h(px(28.)).w(px(70.)).debug_selector(|| "c2".into()))
        });
        let filler = cx.debug_bounds("filler").expect("filler renders");
        assert!(
            filler.size.width > px(20.),
            "empty grow filler should absorb row leftover, got {filler:?}"
        );
    }

    /// Minimal repro of a GPUI layout limitation: a flex item whose content
    /// contains a nested content-sized div (here: `Label`, which renders a
    /// `LabelLike` wrapping a `Div`) does not receive `flex_grow` distribution
    /// on a wrapped flex line, while the same item with a plain text child
    /// does. Raw taffy 0.13 distributes correctly, so the divergence is in
    /// GPUI's div/measurement layer. This is why wrap row-ends use empty
    /// filler strips (see test above) instead of `flex_grow` on the tab
    /// itself. Un-ignore once fixed upstream; see TAB_WRAP_GAPS.md (D4).
    #[gpui::test]
    #[ignore = "known GPUI limitation, repro for upstream issue"]
    fn label_content_breaks_flex_grow_on_wrapped_lines(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let store = settings::SettingsStore::test(cx);
            cx.set_global(store);
            theme_settings::init(theme::LoadThemes::JustBase, cx);
        });

        struct ProbeView;
        impl gpui::Render for ProbeView {
            fn render(
                &mut self,
                _window: &mut gpui::Window,
                _cx: &mut gpui::Context<Self>,
            ) -> impl gpui::IntoElement {
                crate::h_flex()
                    .id("bar")
                    .flex_wrap()
                    .w(gpui::px(300.))
                    .child(
                        div()
                            .id("c0")
                            .h(px(28.))
                            .child(crate::Label::new("aa").single_line())
                            .debug_selector(|| "c0".into()),
                    )
                    .child(
                        div()
                            .id("c1")
                            .h(px(28.))
                            .flex_grow_1()
                            .child(crate::Label::new("bb").single_line())
                            .debug_selector(|| "c1".into()),
                    )
                    .child(
                        div()
                            .id("c2")
                            .h(px(28.))
                            .child("cc")
                            .debug_selector(|| "c2".into()),
                    )
            }
        }

        let (_entity, cx) = cx.add_window_view(|_, _| ProbeView);
        cx.run_until_parked();
        let with_label = cx.debug_bounds("c1").expect("c1 renders");
        let plain_text = cx.debug_bounds("c2").expect("c2 renders");
        assert!(
            with_label.size.width > plain_text.size.width,
            "flex_grow_1 should widen the Label-content item ({:?}) beyond the plain text item ({:?})",
            with_label.size.width,
            plain_text.size.width
        );
    }
}
