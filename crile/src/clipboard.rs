use copypasta::ClipboardProvider;

pub struct Clipboard {
    context: Option<copypasta::ClipboardContext>,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard {
    pub fn new() -> Self {
        Self {
            context: copypasta::ClipboardContext::new()
                .inspect_err(|err| log::error!("Failed to initialize clipboard context, copy and pasting will not work: {err}"))
                .ok(),
        }
    }

    pub fn set(&mut self, text: String) -> bool {
        self.context
            .as_mut()
            .and_then(|context| {
                context
                    .set_contents(text)
                    .inspect_err(|err| log::error!("{err:?}"))
                    .ok()
            })
            .is_some()
    }

    pub fn get(&mut self) -> Option<String> {
        self.context.as_mut().and_then(|context| {
            context
                .get_contents()
                .inspect_err(|err| log::error!("{err:?}"))
                .ok()
        })
    }
}
