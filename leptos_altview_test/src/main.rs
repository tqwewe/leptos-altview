use leptos_altview::view;

fn main() {
    view![div(foo = "bar", name = "ari", on = move |_| {})(
        "hello", "there"
    )];
}
