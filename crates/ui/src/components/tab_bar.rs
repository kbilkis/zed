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
    report_actions_bounds: Option<Rc<dyn Fn(Bounds<Pixels>)>>,
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
            report_actions_bounds: None,
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

    pub fn report_bounds(mut self, report: Rc<dyn Fn(Bounds<Pixels>)>) -> Self {
        self.report_bounds = Some(report);
        self
    }

    pub fn report_actions_bounds(mut self, report: Rc<dyn Fn(Bounds<Pixels>)>) -> Self {
        self.report_actions_bounds = Some(report);
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
            // Rows are planned and built by the pane; the bar stacks them.
            return v_flex()
                .id(self.id)
                .group("tab_bar")
                .relative()
                .overflow_hidden()
                .flex_none()
                .w_full()
                .bg(cx.theme().colors().tab_bar_background)
                .child(
                    div()
                        .absolute()
                        .bottom_0()
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
                .children(self.children)
                .when(!self.end_children.is_empty(), |this| {
                    // Container keeps painting even unfocused (its bottom
                    // border completes the bar's); the pane hides only the
                    // buttons.
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
                            .when_some(self.report_actions_bounds, |this, report| {
                                this.child(
                                    canvas(
                                        move |bounds: Bounds<Pixels>,
                                              _: &mut Window,
                                              _: &mut App| {
                                            report(bounds)
                                        },
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
    use crate::prelude::*;
    use gpui::{TestAppContext, div, px, size};

    #[gpui::test]
    fn canvas_fires_inside_invisible(cx: &mut TestAppContext) {
        use gpui::canvas;
        use std::cell::Cell;
        use std::rc::Rc;
        let fired = Rc::new(Cell::new(0));
        let cx = cx.add_empty_window();
        cx.draw(gpui::Point::default(), size(px(300.), px(100.)), |_, _| {
            let fired = fired.clone();
            div().id("root").child(
                h_flex()
                    .id("hidden_actions")
                    .invisible()
                    .h(px(28.))
                    .px_2()
                    .child(div().w(px(60.)).h(px(28.)))
                    .child(
                        canvas(
                            move |_: gpui::Bounds<gpui::Pixels>,
                                  _: &mut gpui::Window,
                                  _: &mut gpui::App| {
                                fired.set(fired.get() + 1);
                            },
                            |_: gpui::Bounds<gpui::Pixels>,
                             _: (),
                             _: &mut gpui::Window,
                             _: &mut gpui::App| {},
                        )
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full(),
                    ),
            )
        });
        assert!(
            fired.get() > 0,
            "canvas inside invisible subtree should still prepaint, fired {}",
            fired.get()
        );
    }

    #[gpui::test]
    fn absolute_child_reports_padding_box(cx: &mut TestAppContext) {
        let cx = cx.add_empty_window();
        cx.draw(gpui::Point::default(), size(px(300.), px(100.)), |_, _| {
            crate::h_flex()
                .id("bar")
                .relative()
                .w_full()
                .pr(px(80.))
                .h(px(28.))
                .child(
                    div()
                        .id("probe")
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .debug_selector(|| "probe".into()),
                )
                .child(
                    div()
                        .id("content")
                        .w(px(50.))
                        .h(px(28.))
                        .debug_selector(|| "content".into()),
                )
        });
        let probe = cx.debug_bounds("probe").expect("probe renders");
        assert!(
            probe.size.width > px(250.),
            "absolute size_full child should span the padding box (~300), got {probe:?}"
        );
    }
}
