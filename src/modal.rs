use std::{cell::RefCell, rc::Rc};

use indexmap::IndexMap;
use leptos::*;
use uuid::Uuid;

// TODO: Add mount prop.
// TODO: Add dialog component, making modal the underlying technology?

#[derive(Clone)]
pub struct ModalData {
    pub id: Uuid,
    pub children: ModalChildren,
}

#[derive(Clone)]
pub enum ModalChildren {
    Once(View),
    Dynamic(Rc<ChildrenFn>, Scope),
}

#[derive(Copy, Clone)]
pub struct ModalRootContext {
    pub modals: ReadSignal<IndexMap<Uuid, ModalData>>,
    pub set_modals: WriteSignal<IndexMap<Uuid, ModalData>>,
}

#[component]
pub fn ModalRoot(cx: Scope, children: Children) -> impl IntoView {
    let (modals, set_modals) = create_signal(cx, IndexMap::new());
    provide_context::<ModalRootContext>(cx, ModalRootContext { modals, set_modals });
    view! { cx,
        { children(cx) }

        <leptonic-modal-host>
            <Show fallback=|_cx| view! { cx, } when=move || modals.get().len() != 0>
                <leptonic-modal-backdrop></leptonic-modal-backdrop>

                <leptonic-modals>
                    {move || modals.get().last().map(|(_, modal)| view! { cx,
                        <leptonic-modal>
                            { match &modal.children {
                                ModalChildren::Once(view) => view.clone(),
                                ModalChildren::Dynamic(children, cx) => children(*cx).into_view(*cx)
                            } }
                        </leptonic-modal>
                    })}
                </leptonic-modals>
            </Show>
        </leptonic-modal-host>
    }
}

#[component]
pub fn Modal(
    cx: Scope,
    #[prop(into)] show_when: MaybeSignal<bool>,
    children: Children,
) -> impl IntoView {
    let modals = use_context::<ModalRootContext>(cx).unwrap();
    let children = children(cx).into_view(cx); // TODO: Is it ok to build this view once?

    let id = Uuid::new_v4();

    create_effect(cx, move |_| match show_when.get() {
        true => modals.set_modals.update(|modals| {
            modals.insert(
                id,
                ModalData {
                    id,
                    children: ModalChildren::Once(children.clone()),
                },
            );
        }),
        false => modals.set_modals.update(|modals| {
            modals.remove(&id);
        }),
    });

    on_cleanup(cx, move || {
        tracing::info!("cleanup modal");
    });

    // Intentionally empty, as children are rendered using the modal root.
    view! { cx,
    }
}

// TODO: Show a modal in a different scope. This way, not including a shown modal anymore would remove it.
#[component]
pub fn ModalFn(
    cx: Scope,
    #[prop(into)] show_when: MaybeSignal<bool>,
    children: ChildrenFn,
) -> impl IntoView {
    let children = Rc::new(children);

    let child_scope = RefCell::new(Option::<Scope>::None);

    let router = move || {
        // Whenever the "show_when" changes, the modal must be re-rendered in a new scope.
        let show: bool = show_when.get();

        let (view, _) = cx.run_child_scope(|cx| {
            let prev_cx = std::mem::replace(&mut *child_scope.borrow_mut(), Some(cx));
            if let Some(prev_cx) = prev_cx {
                prev_cx.dispose();
            }

            let modals = use_context::<ModalRootContext>(cx).unwrap();
            let id = Uuid::new_v4();

            let c_clone = children.clone();

            //if show {
            modals.set_modals.update(|modals| {
                modals.insert(
                    id,
                    ModalData {
                        id,
                        children: ModalChildren::Dynamic(c_clone.clone(), cx),
                    },
                );
            });

            on_cleanup(cx, move || {
                tracing::warn!("dispose");
                modals.set_modals.update(|modals| {
                    modals.remove(&id);
                })
            });
            //}

            // content(cx, show).into_view(cx)
            // Intentionally empty, as children are rendered using the modal root.
            view! { cx,
            }
        });

        view
    };

    router
}

#[component]
pub fn ModalHeader(cx: Scope, children: Children) -> impl IntoView {
    view! { cx,
        <leptonic-modal-header>
            { children(cx) }
        </leptonic-modal-header>
    }
}

#[component]
pub fn ModalTitle(cx: Scope, children: Children) -> impl IntoView {
    view! { cx,
        <leptonic-modal-title>
            { children(cx) }
        </leptonic-modal-title>
    }
}

#[component]
pub fn ModalBody(cx: Scope, children: Children) -> impl IntoView {
    view! { cx,
        <leptonic-modal-body>
            { children(cx) }
        </leptonic-modal-body>
    }
}

#[component]
pub fn ModalFooter(cx: Scope, children: Children) -> impl IntoView {
    view! { cx,
        <leptonic-modal-footer>
            { children(cx) }
        </leptonic-modal-footer>
    }
}
