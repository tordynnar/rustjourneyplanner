use leptonic::prelude::*;
use leptos::*;
use web_sys;
use tracing;
use uuid;
use leptonic::select::SelectOption;
use leptos_icons::BsIcon;
use web_sys::{HtmlElement, KeyboardEvent};


#[component]
pub fn CustomOptionalSelect<O>(
    #[prop(into)] options: MaybeSignal<Vec<O>>,
    #[prop(into)] selected: Signal<Option<O>>,
    #[prop(into)] set_selected: Out<Option<O>>,
    #[prop(into)] render_option: ViewCallback<O>,
    #[prop(into)] allow_deselect: MaybeSignal<bool>,
    #[prop(into, optional)] search_text_provider: Option<Callback<O, String>>,
    #[prop(into, optional)] filter_provider: Option<Callback<(String, Vec<O>), Vec<O>>>,
    #[prop(into, optional)] autofocus_search: Option<Signal<bool>>,
    #[prop(into, optional)] class: Option<AttributeValue>,
    #[prop(into, optional)] style: Option<AttributeValue>,
) -> impl IntoView
where
    O: SelectOption + 'static,
{
    let render_option = StoredValue::new(render_option);

    let id: uuid::Uuid = uuid::Uuid::new_v4();
    let id_string = format!("s-{id}");
    let id_selector_string = format!("#{id_string}");

    let (focused, set_focused) = create_signal(false);
    let (show_options, set_show_options) = create_signal(false);

    let autofocus_search =
        autofocus_search.unwrap_or(expect_context::<Leptonic>().is_desktop_device);

    let search_should_be_focused =
        Signal::derive(move || show_options.get() && autofocus_search.get());
    let (search_is_focused, set_search_is_focused) = create_signal(false);

    let stored_options = store_value(options);
    let (preselected, set_preselected) = create_signal(Option::<O>::None);
    let memoized_preselected = create_memo(move |_| preselected.get());

    let (search, set_search) = create_signal("".to_owned());

    let search_text_provider = search_text_provider.unwrap_or(Callback::new(move |o : O| format!("{:?}", o)));

    let filter_provider = filter_provider.unwrap_or(Callback::new(move |(s, o) : (String, Vec<O>)| {
        let lowercased_search = s.to_lowercase();
        o.into_iter()
            .filter(|it| {
                search_text_provider.call(it.clone())
                    .to_lowercase()
                    .contains(lowercased_search.as_str())
            })
            .collect::<Vec<_>>()
    }));

    let filtered_options = create_memo(move |_| {
        filter_provider.call((search.get(), stored_options.get_value().get()))
    });

    let has_options = create_memo(move |_| !filtered_options.with(|options| options.is_empty()));

    let set_selected_clone = set_selected.clone();

    let select = StoredValue::new(Callback::new(move |option: O| {
        set_selected.set(Some(option));
        set_show_options.set(false);
    }));

    let deselect = move || {
        set_selected_clone.set(None);
    };

    let is_selected = move |option: &O| selected.with(|selected| selected.as_ref() == Some(option));

    let is_disabled = move |option: &O| selected.with(|selected| selected.as_ref() == Some(option));

    let is_disabled_untracked =
        move |option: &O| selected.with_untracked(|selected| selected.as_ref() == Some(option));

    // We need to check for global mouse events.
    // If our option list is shown and such an event occurs and does not target our option list, the options list should be closed.
    create_click_away_listener(
        id_selector_string.clone(),
        show_options,
        set_show_options.into(),
    );

    create_key_down_listener(move |e| {
        match (show_options.get_untracked(), focused.get_untracked()) {
            (true, _) => match e.key().as_str() {
                "Escape" => set_show_options.set(false),
                "Backspace" => {
                    if !search_is_focused.get_untracked() {
                        set_show_options.set(false)
                    }
                }
                "ArrowUp" => {
                    e.prevent_default();
                    e.stop_propagation();
                    // TODO: Use options_available_for_preselect.with_untracked when https://github.com/leptos-rs/leptos/issues/1212 is resolved and released.
                    select_previous(
                        &filtered_options.get_untracked(),
                        preselected,
                        set_preselected,
                    );
                }
                "ArrowDown" => {
                    e.prevent_default();
                    e.stop_propagation();
                    // TODO: Use options_available_for_preselect.with_untracked when https://github.com/leptos-rs/leptos/issues/1212 is resolved and released.
                    select_next(
                        &filtered_options.get_untracked(),
                        preselected,
                        set_preselected,
                    );
                }
                "Enter" => {
                    e.prevent_default();
                    e.stop_propagation();
                    if let Some(preselected) = preselected.get_untracked() {
                        if !is_disabled_untracked(&preselected) {
                            select.get_value().call(preselected)
                        }
                    }
                }
                _ => {}
            },
            (false, true) => match e.key().as_str() {
                "Enter" | "ArrowDown" => {
                    e.prevent_default();
                    e.stop_propagation();
                    set_show_options.set(true);
                }
                _ => {}
            },
            _ => {}
        }
    });

    let toggle_show = move || set_show_options.update(|val| *val = !*val);

    let wrapper: NodeRef<html::Div> = create_node_ref();

    // Put focus back on our wrapper when the dropdown was closed while the search input had focus.
    create_effect(move |_| {
        if !show_options.get() && search_is_focused.get_untracked() {
            // TODO: Use with() when available.
            if let Some(wrapper) = wrapper.get() {
                wrapper.focus().unwrap();
            } else {
                tracing::warn!("missing node_ref");
            }
        }
    });

    view! {
        // TODO: If possible, move this focus-tracking functionality to our main leptonic-select element. it requires the focus() method to be available.
        <div
            node_ref=wrapper
            class="leptonic-select-wrapper"
            tabindex=0
            on:blur=move |_| set_focused.set(false)
            on:focus=move |_| set_focused.set(true)
        >
            <leptonic-select
                id=id_string
                data-variant="optional-select"
                aria-haspopup="listbox"
                class=class
                style=style
            >
                <leptonic-select-selected on:click=move |_| toggle_show()>
                    { move || match selected.get().clone() {
                        None => ().into_view(),
                        Some(selected) => view! {
                            <leptonic-select-option>
                                { render_option.get_value().call(selected) }
                            </leptonic-select-option>
                        }.into_view(),
                    }}

                    { match allow_deselect.get() {
                        false => ().into_view(),
                        true => view! {
                            <leptonic-select-deselect-trigger on:click=move |e| {
                                e.prevent_default();
                                e.stop_propagation();
                                deselect();
                            }>
                                <Icon icon=BsIcon::BsXCircleFill/>
                            </leptonic-select-deselect-trigger>
                        }.into_view(),
                    }}

                    <leptonic-select-show-trigger>
                        {move || match show_options.get() {
                            true => view! { <Icon icon=BsIcon::BsCaretUpFill/>},
                            false => view! { <Icon icon=BsIcon::BsCaretDownFill/>}
                        }}
                    </leptonic-select-show-trigger>
                </leptonic-select-selected>

                <leptonic-select-options class:shown=move || show_options.get()>
                    <TextInput
                        get=search
                        set=set_search
                        should_be_focused=search_should_be_focused
                        on_focus_change=move |focused| {
                            // We only update our state as long as show_options is true.
                            // It it is no longer true, the dropdown is no longer shown through a CSS rule (display: none).
                            // This will automatically de-focus the search input if it had focus, resulting in a call of this callback.
                            // When storing the received `false` in `search_is_focused` before our effect above, resetting focus on our wrapper may, runs,
                            // that create_effect will not be able to set the focus. We accept not setting `search_is_focused` all the time
                            // for the create_effect above to work reliably.
                            if show_options.get_untracked() {
                                set_search_is_focused.set(focused);
                            }
                        }
                        class="search"
                    />

                    <Show
                        when=move || show_options.get()
                        fallback=move || ()
                    >
                        // TOD: Use <For> once leptos 0.4 is out. Use full option for hash.
                        { filtered_options.get().into_iter().map(|option| {
                            let clone1 = option.clone();
                            let clone2 = option.clone();
                            let clone3 = option.clone();
                            let clone4 = option.clone();
                            let clone5 = option.clone();
                            view! {
                                <leptonic-select-option
                                    class:preselected=move || memoized_preselected.with(|preselected| preselected.as_ref() == Some(&option))
                                    class:selected=move || is_selected(&clone4)
                                    class:disabled=move || is_disabled(&clone5)
                                    on:mouseenter=move |_e| {
                                        set_preselected.set(Some(clone3.clone()));
                                    }
                                    on:click=move |_e| {
                                        if !is_disabled_untracked(&clone2) {
                                            select.get_value().call(clone2.clone())
                                        }
                                    }
                                >
                                    { render_option.get_value().call(clone1) }
                                </leptonic-select-option>
                            }
                        }).collect_view() }

                        { move || match has_options.get() {
                            true => ().into_view(),
                            false => view! {
                                <div class="option">
                                    "No options..."
                                </div>
                            }.into_view(),
                        } }
                    </Show>
                </leptonic-select-options>
            </leptonic-select>
        </div>
    }
}


fn create_click_away_listener(
    id_selector_string: String,
    when: ReadSignal<bool>,
    on_click_outside: Out<bool>,
) {
    let g_mouse_event =
        use_context::<GlobalClickEvent>().expect("Must be a child of the Root component.");

    create_effect(move |_old| {
        use wasm_bindgen::JsCast;
        let last_mouse_event = g_mouse_event.read_signal.get();

        if when.get_untracked() {
            if let Some(e) = last_mouse_event {
                if let Some(target) = e.target() {
                    if let Some(target_elem) = target.dyn_ref::<HtmlElement>() {
                        match target_elem.closest(id_selector_string.as_ref()) {
                            Ok(closest) => {
                                if let Some(_found) = closest {
                                    // User clicked on the options list. Ignoring this global mouse event.
                                } else {
                                    // User clicked outside.
                                    on_click_outside.set(false);
                                }
                            }
                            Err(err) => {
                                tracing::error!("Error processing latest mouse event: {err:?}");
                            }
                        }
                    }
                }
            }
        }
    });
}

fn select_previous<O: SelectOption + 'static>(
    available: &[O],
    preselected: ReadSignal<Option<O>>,
    set_preselected: WriteSignal<Option<O>>,
) {
    let previous = preselected.with_untracked(|current| match current {
        Some(current) => match available.iter().position(|it| it == current) {
            Some(current_pos) => match current_pos >= 1 {
                true => Some(available[current_pos - 1].clone()),
                false => available.last().cloned(),
            },
            None => available.last().cloned(),
        },
        None => available.last().cloned(),
    });
    set_preselected.set(previous);
}

fn select_next<O: SelectOption + 'static>(
    available: &[O],
    preselected: ReadSignal<Option<O>>,
    set_preselected: WriteSignal<Option<O>>,
) {
    let next = preselected.with_untracked(|current| match current {
        Some(current) => match available.iter().position(|it| it == current) {
            Some(current_pos) => match (current_pos + 1) < available.len() {
                true => Some(available[current_pos + 1].clone()),
                false => available.first().cloned(),
            },
            None => available.first().cloned(),
        },
        None => available.first().cloned(),
    });
    set_preselected.set(next);
}

fn create_key_down_listener<T: Fn(KeyboardEvent) + 'static>(then: T) {
    let g_keyboard_event =
        use_context::<GlobalKeyboardEvent>().expect("Must be a child of the Root component.");

    create_effect(move |_old| {
        let g_keyboard_event = g_keyboard_event.read_signal.get();
        if let Some(e) = g_keyboard_event {
            then(e);
        }
    });
}
