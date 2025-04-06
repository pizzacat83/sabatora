use saba_core::renderer::css::parser::parse_css_stylesheet;
use saba_core::renderer::dom::node::{Element, ElementKind, NodeData};
use saba_core::renderer::html::parser::HtmlParser;
use saba_core::renderer::html::token::HtmlTokenizer;
use saba_core::renderer::layout::box_tree::{
    construct_box_tree, BlockBoxChildren, InlineBox, InlineBoxData,
};
use saba_core::renderer::layout::layout_view::{get_style_content, LayoutView};
use saba_core::renderer::layout::{
    box_tree::{BlockBox, BlockBoxData},
    computed_style::ComputedStyle,
    computed_style::DisplayType,
};
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let textarea_ref = use_node_ref();

    let box_tree = use_state(|| None);

    let on_submit = {
        let textarea_ref = textarea_ref.clone();
        let box_tree = box_tree.clone();
        Callback::from(move |_| {
            let html = textarea_ref.cast::<HtmlTextAreaElement>().unwrap().value();

            box_tree.set(Some(construct_box_tree_from_html(html)));
        })
    };

    html! {
        <>
            <textarea placeholder="HTML here..." ref={textarea_ref.clone()} />
            <button onclick={on_submit}>{"Visualize"}</button>
            {box_tree.as_ref().map(|box_tree| {
                html! {
                    <BlockBoxC block_box={box_tree.clone()} />
                }
            })}
        </>
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
fn construct_box_tree_from_html(html: String) -> BlockBox {
    let t = HtmlTokenizer::new(html);
    let window = HtmlParser::new(t).construct_tree();
    let dom = window.borrow().document();
    let style = get_style_content(dom.clone());
    let cssom = parse_css_stylesheet(style);
    let layout_view = LayoutView::layout(dom, &cssom);
    construct_box_tree(layout_view)
}
