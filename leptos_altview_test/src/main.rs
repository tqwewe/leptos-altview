use leptos_altview::view;

fn main() {
    view![div(
        class = "hi",
        foo = "bar",
        name = "ari",
        style = "width: 100%" // on = move |_| {}
    )("hello", "there")];
}
