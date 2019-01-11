use parsley::prelude::*;
use stdweb::*;
use yew::prelude::*;

pub struct Terminal {
    context: Context,
    history: String,
    value: String,
}

pub enum Msg {
    GotInput(String),
    KeyUp(String),
    Clicked,
}

impl Component for Terminal {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Terminal {
            context: Context::base(),
            history: String::with_capacity(99999),
            value: String::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Clicked => {
                js! {
                    document.getElementById("theTerminalInput").focus();
                }
                false
            }
            Msg::KeyUp(ref s) if s == "Enter" && !self.value.is_empty() => {
                // save command
                self.history.push_str(&format!("\n> {}", self.value));
                // evaluate
                match run_in(&self.value, &mut self.context) {
                    Ok(result) => {
                        // save result, if it's not empty
                        let res = format!("{}", result);
                        if !res.is_empty() {
                            self.history.push_str(&format!("\n{}", res));
                        }
                    }
                    Err(error) => {
                        // save error
                        self.history.push_str(&format!("\n{}", error));
                    }
                }
                self.value = String::new();
                true
            }
            Msg::GotInput(s) => {
                self.value = s;
                true
            }
            _ => false,
        }
    }
}

impl Renderable<Terminal> for Terminal {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="Terminal", onclick=|_| Msg::Clicked, >
                <div class="History", >
                    { &self.history }
                </div>
                <div class="InputLine", >
                    { "> " }
                    <input
                        id="theTerminalInput",
                        placeholder="Enter an expression...",
                        oninput=|e| Msg::GotInput(e.value),
                        onkeyup=|e| Msg::KeyUp(e.code()),
                        value=&self.value,
                    />
                </div>
            </div>
        }
    }
}
