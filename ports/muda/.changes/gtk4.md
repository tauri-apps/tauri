---
"muda": minor
---

Port the Linux backend from GTK3 to GTK4 (`gtk4` 0.11, `glib` 0.22). Every public method keeps its name and signature shape; the gtk types in returns and bounds follow the toolkit (`PopoverMenu`/`PopoverMenuBar`, `Widget` bounds), and `ContextMenu::gtk_context_menu` is retained on GTK4. Raises MSRV to 1.92.
