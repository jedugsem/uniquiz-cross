use uniquiz::Controls;
fn main() -> iced::Result {
    iced::application(Controls::default, Controls::update, Controls::view)
        .title(Controls::title)
        .run()
}
