use saba_core::renderer::dom::node::{Element, ElementKind, NodeData};
use saba_core::renderer::layout::box_tree::{BlockBoxChildren, InlineBox, InlineBoxData};
use saba_core::renderer::layout::{
    box_tree::{BlockBox, BlockBoxData},
    computed_style::ComputedStyle,
    computed_style::DisplayType,
};
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <BlockBoxC />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(BlockBoxC)]
fn block_box() -> Html {
    html! {
        <div style="border: 1px solid black;">
            <div style="border-bottom: 1px solid black; padding: 0.2rem 1rem; background-color: #eee;">{"body"}</div>
            // <div class="block-box-children">children</div>
        </div>
    }
}

// TODO: use user-provided html
fn construct_box_tree() -> BlockBox {
    let expected = BlockBox {
        data: BlockBoxData::Element(Element::new(ElementKind::Body)),
        style: ComputedStyle {
            display: Some(DisplayType::Block),
        },
        children: BlockBoxChildren::Blocks(vec![
            BlockBox {
                data: BlockBoxData::Anonymous,
                style: ComputedStyle {
                    display: Some(DisplayType::Block),
                },
                children: BlockBoxChildren::Inlines(vec![
                    InlineBox {
                        data: InlineBoxData::Element(Element::new(ElementKind::A)),
                        style: ComputedStyle {
                            display: Some(DisplayType::Inline),
                        },
                        text: Some("inline1 inline1 inline1".into()),
                        children: vec![],
                    },
                    InlineBox {
                        data: InlineBoxData::Anonymous,
                        style: ComputedStyle {
                            display: Some(DisplayType::Inline),
                        },
                        text: Some("inline1 inline1 inline1".into()),
                        children: vec![],
                    },
                    InlineBox {
                        data: InlineBoxData::Element(Element::new(ElementKind::A)),
                        style: ComputedStyle {
                            display: Some(DisplayType::Inline),
                        },
                        text: Some("inline3 inline3 inline3".into()),
                        children: vec![],
                    },
                ]),
            },
            BlockBox {
                data: BlockBoxData::Element(Element::new(ElementKind::P)),
                style: ComputedStyle {
                    display: Some(DisplayType::Block),
                },
                children: BlockBoxChildren::Inlines(vec![InlineBox {
                    data: InlineBoxData::Anonymous,
                    style: ComputedStyle {
                        display: Some(DisplayType::Inline),
                    },
                    text: Some("block4 block4 block4".into()),
                    children: vec![],
                }]),
            },
            BlockBox {
                data: BlockBoxData::Element(Element::new(ElementKind::P)),
                style: ComputedStyle {
                    display: Some(DisplayType::Block),
                },
                children: BlockBoxChildren::Inlines(vec![InlineBox {
                    data: InlineBoxData::Anonymous,
                    style: ComputedStyle {
                        display: Some(DisplayType::Inline),
                    },
                    text: Some("block4 block4 block4".into()),
                    children: vec![],
                }]),
            },
            BlockBox {
                data: BlockBoxData::Anonymous,
                style: ComputedStyle {
                    display: Some(DisplayType::Block),
                },
                children: BlockBoxChildren::Inlines(vec![InlineBox {
                    data: InlineBoxData::Anonymous,
                    style: ComputedStyle {
                        display: Some(DisplayType::Inline),
                    },
                    text: Some("inline6 inline6 inline6".into()),
                    children: vec![],
                }]),
            },
        ]),
    };

    expected
}
