use iced::Task;
use uniquiz::Controls;
use uniquiz::Message;
fn main() -> iced::Result {
    iced::application(
        || {
            (
                Controls::default(),
                Task::perform(async {}, |_| Message::Boot),
            )
        },
        Controls::update,
        Controls::view,
    )
    .title(Controls::title)
    .run()
}
