use iced::widget::button;
use iced::Border;
use iced_material::theme;
use iced_widget::{button::Button, container, row, text_editor, Space};
pub mod card;

pub fn searchbar<'a, Message, Renderer>(
    content: &'a text_editor::Content<Renderer>,
    f: impl Fn(text_editor::Action) -> Message + 'a,
    fc: Option<impl Fn() -> Message + 'a>,
) -> iced::Element<'a, Message, theme::Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced_core::Renderer + iced_core::text::Renderer,
{
    let editor: iced::Element<Message, _, _> = text_editor(content).on_action(f).into();
    let cont = if let Some(fcc) = fc {
        row!(editor, Space::with_width(5), button("📋").on_press(fcc())).into()
    } else {
        editor
    };
    container(cont)
        .padding(5)
        .style(|theme: &theme::Theme| container::Style {
            background: Some(iced::Background::Color(theme.colors().background.darkest)),
            border: Border::default().rounded(10),
            ..Default::default()
        })
        .width(500)
        .into()
}

pub fn togg<'a, Message, Renderer>(
    content: impl Into<iced::Element<'a, Message, theme::Theme, Renderer>>,
    state: bool,
    f: impl Fn(bool) -> Message + 'a,
) -> Button<'a, Message, theme::Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced_core::Renderer + iced_core::text::Renderer,
{
    button(content)
        .style(if state {
            theme::button::selected
        } else {
            theme::button::unselected
        })
        .on_press(f(!state))
}
