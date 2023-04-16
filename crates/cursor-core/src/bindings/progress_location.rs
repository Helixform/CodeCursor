use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = RustProgressLocation)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressLocation {
    SourceControl = 1,

    /**
     * Show progress in the status bar of the editor. Neither supports cancellation nor discrete progress.
     * Supports rendering of {@link ThemeIcon theme icons} via the `$(<name>)`-syntax in the progress label.
     */
    Window = 10,

    /**
     * Show progress as notification with an optional cancel button. Supports to show infinite and discrete
     * progress but does not support rendering of icons.
     */
    Notification = 15,
}
