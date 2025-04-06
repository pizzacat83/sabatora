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
    let box_tree = construct_box_tree();

    html! {
        <BlockBoxC block_box={box_tree} />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

#[derive(Properties, PartialEq)]
struct BlockBoxProps {
    block_box: BlockBox,
}

#[function_component(BlockBoxC)]
fn block_box(props: &BlockBoxProps) -> Html {
    let title = match props.block_box.data {
        BlockBoxData::Element(ref element) => format!("<{}>", element.tag_name()),
        BlockBoxData::Anonymous => "anonymous".to_string(),
    };

    let children = match props.block_box.children {
        BlockBoxChildren::Blocks(ref blocks) => blocks
            .iter()
            .map(|block| html! { <BlockBoxC block_box={block.clone()} /> })
            .collect::<Html>(),
        BlockBoxChildren::Inlines(ref inlines) => inlines
            .iter()
            .map(|inline| html! { <InlineBoxC inline_box={inline.clone()} /> })
            .collect::<Html>(),
        BlockBoxChildren::Empty => html! {},
    };

    let children_style = format!(
        "padding: 1rem; display: flex; gap: 1rem; {}",
        match props.block_box.children {
            BlockBoxChildren::Blocks(_) => "flex-direction: column;",
            BlockBoxChildren::Inlines(_) => "flex-direction: row;",
            BlockBoxChildren::Empty => "",
        }
    );

    html! {
        <div style="border: 1px solid black; text-wrap-mode: nowrap; min-width: fit-content;">
            <div style="border-bottom: 1px solid black; padding: 0.2rem 1rem; background-color: #eee;">{title}</div>
            <div style={children_style}>{children}</div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct InlineBoxProps {
    inline_box: InlineBox,
}

#[function_component(InlineBoxC)]
fn inline_box(props: &InlineBoxProps) -> Html {
    let title = match props.inline_box.data {
        InlineBoxData::Element(ref element) => format!("<{}>", element.tag_name()),
        InlineBoxData::Anonymous => "anonymous".to_string(),
    };

    let children = html! {
        <>
            {props.inline_box.text.clone()}
            {
                props.inline_box.children.iter().map(|child| {
                    html! { <InlineBoxC inline_box={child.clone()} /> }
                }).collect::<Html>()
            }
        </>
    };

    html! {
        <div style="display: inline-block; border: 1px solid black; border-radius: 0.5rem;">
            <div style="border-bottom: 1px solid black; padding: 0.2rem 1rem; background-color: #eee; border-radius: 0.5rem 0.5rem 0 0;">{title}</div>
            <div style="padding: 1rem">{children}</div>
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
                        text: Some("inline2 inline2 inline2".into()),
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
