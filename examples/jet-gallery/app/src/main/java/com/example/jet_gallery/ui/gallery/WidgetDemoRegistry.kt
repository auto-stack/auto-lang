package com.example.jet_gallery.ui.gallery

const val EXPECTED_WIDGET_COUNT = 51

private fun demo(
    id: String,
    title: String,
    category: WidgetCategory,
    stdlibPath: String,
    supportTier: SupportTier,
    description: String,
    composeTarget: String,
    vararg notes: String,
): WidgetDemo = WidgetDemo(
    id = id,
    title = title,
    category = category,
    stdlibPath = stdlibPath,
    supportTier = supportTier,
    description = description,
    composeTarget = composeTarget,
    notes = notes.toList(),
)

object WidgetDemoRegistry {
    val excludedWebOnlyPages = setOf(
        "carousel",
        "combobox",
        "command",
        "datepicker",
        "index",
        "label",
        "toggle",
        "togglegroup",
    )

    val demos = listOf(
        demo("col", "Col", WidgetCategory.Layout, "stdlib/aura/widgets/layout/Col.at", SupportTier.Native, "Vertical layout container.", "Column + verticalArrangement", "Gap maps to spaced arrangements."),
        demo("row", "Row", WidgetCategory.Layout, "stdlib/aura/widgets/layout/Row.at", SupportTier.Native, "Horizontal layout container.", "Row + horizontalArrangement", "Weight and alignment behavior matter for a2jet."),
        demo("center", "Center", WidgetCategory.Layout, "stdlib/aura/widgets/layout/Center.at", SupportTier.Native, "Centers content in both axes.", "Box(contentAlignment = Alignment.Center)", "Usually emitted as a Box wrapper."),
        demo("card", "Card", WidgetCategory.Layout, "stdlib/aura/widgets/layout/Card.at", SupportTier.Native, "Elevated content container.", "Card / ElevatedCard / OutlinedCard", "Variants should line up with AURA card styles."),
        demo("scrollarea", "ScrollArea", WidgetCategory.Layout, "stdlib/aura/widgets/layout/ScrollArea.at", SupportTier.Native, "Scrollable viewport.", "Column.verticalScroll / LazyColumn", "Choose lazy vs eager based on content shape."),
        demo("aspectratio", "AspectRatio", WidgetCategory.Layout, "stdlib/aura/widgets/layout/AspectRatio.at", SupportTier.Native, "Keeps media at a stable ratio.", "Modifier.aspectRatio()", "Common for media cards."),
        demo("collapsible", "Collapsible", WidgetCategory.Layout, "stdlib/aura/widgets/layout/Collapsible.at", SupportTier.Composite, "Expandable content block.", "AnimatedVisibility + stateful trigger", "Compose does not have a single built-in collapsible widget."),
        demo("accordion", "Accordion", WidgetCategory.Layout, "stdlib/aura/widgets/layout/Accordion.at", SupportTier.Composite, "Stacked collapsible sections.", "List of expandable cards", "Useful as a reference for structured interactive lists."),

        demo("button", "Button", WidgetCategory.Form, "stdlib/aura/widgets/form/Button.at", SupportTier.Native, "Primary action control.", "Button / OutlinedButton / TextButton", "Variant mapping is important for a2jet."),
        demo("input", "Input", WidgetCategory.Form, "stdlib/aura/widgets/form/Input.at", SupportTier.Native, "Single-line text input.", "OutlinedTextField / TextField", "State and placeholder behavior must be preserved."),
        demo("checkbox", "Checkbox", WidgetCategory.Form, "stdlib/aura/widgets/form/Checkbox.at", SupportTier.Native, "Boolean toggle with label.", "Checkbox + Row wrapper", "On Android the label is usually composed around the checkbox."),
        demo("switch", "Switch", WidgetCategory.Form, "stdlib/aura/widgets/form/Switch.at", SupportTier.Native, "Boolean switch for settings.", "Switch", "Supports enabled and checked states."),
        demo("select", "Select", WidgetCategory.Form, "stdlib/aura/widgets/form/Select.at", SupportTier.Composite, "Choice selection field.", "Exposed dropdown or anchored menu composite", "No single Material3 composable fully matches shadcn-style select."),
        demo("slider", "Slider", WidgetCategory.Form, "stdlib/aura/widgets/form/Slider.at", SupportTier.Native, "Continuous numeric selection.", "Slider", "Discrete steps can be layered later if needed."),
        demo("radiogroup", "RadioGroup", WidgetCategory.Form, "stdlib/aura/widgets/form/RadioGroup.at", SupportTier.Native, "Mutually exclusive options.", "RadioButton list", "Selection state should stay centralized."),
        demo("textarea", "Textarea", WidgetCategory.Form, "stdlib/aura/widgets/form/Textarea.at", SupportTier.Native, "Multi-line text editor.", "OutlinedTextField with minLines/maxLines", "Compose uses the same text field family."),
        demo("form", "Form", WidgetCategory.Form, "stdlib/aura/widgets/form/Form.at", SupportTier.Composite, "Labeled grouped input flow.", "Column + field blocks + validation state", "This is a higher-level pattern, not a single Compose widget."),

        demo("text", "Text", WidgetCategory.Display, "stdlib/aura/widgets/display/Text.at", SupportTier.Native, "Typography primitive.", "Text + Material typography", "Style tokens matter for parity."),
        demo("image", "Image", WidgetCategory.Display, "stdlib/aura/widgets/display/Image.at", SupportTier.Native, "Bitmap or remote media presentation.", "Image / AsyncImage", "Resource and network loading are both relevant."),
        demo("badge", "Badge", WidgetCategory.Display, "stdlib/aura/widgets/display/Badge.at", SupportTier.Native, "Small count or status marker.", "Badge / BadgedBox", "Often paired with nav items and actions."),
        demo("avatar", "Avatar", WidgetCategory.Display, "stdlib/aura/widgets/display/Avatar.at", SupportTier.Composite, "User identity image or initials.", "AsyncImage + CircleShape + fallback", "Compose needs a small fallback pattern."),
        demo("separator", "Separator", WidgetCategory.Display, "stdlib/aura/widgets/display/Separator.at", SupportTier.Native, "Visual divider between regions.", "HorizontalDivider / VerticalDivider", "Divider thickness and color should remain theme-driven."),
        demo("skeleton", "Skeleton", WidgetCategory.Display, "stdlib/aura/widgets/display/Skeleton.at", SupportTier.Composite, "Loading placeholder state.", "Animated placeholder surfaces", "Material3 does not ship a first-party skeleton composable."),
        demo("swiper", "Swiper", WidgetCategory.Display, "stdlib/aura/widgets/display/Swiper.at", SupportTier.Composite, "Swipeable pager/carousel.", "HorizontalPager + pager state + indicators", "Auto-play can be layered later when generator support lands."),

        demo("tabs", "Tabs", WidgetCategory.Navigation, "stdlib/aura/widgets/navigation/Tabs.at", SupportTier.Native, "Segmented content navigation.", "TabRow + Tab + selected index", "One of the cleaner direct mappings."),
        demo("breadcrumb", "Breadcrumb", WidgetCategory.Navigation, "stdlib/aura/widgets/navigation/Breadcrumb.at", SupportTier.Composite, "Hierarchical path display.", "Row of clickable text and separators", "Mostly a layout/text pattern."),
        demo("navigationmenu", "NavigationMenu", WidgetCategory.Navigation, "stdlib/aura/widgets/navigation/NavigationMenu.at", SupportTier.Composite, "Grouped navigation affordances.", "Row/rail of chips or menu buttons", "This is more of a navigation pattern than a single widget."),
        demo("pagination", "Pagination", WidgetCategory.Navigation, "stdlib/aura/widgets/navigation/Pagination.at", SupportTier.Composite, "Paged list navigation controls.", "Button row + selected page state", "Useful for table-style flows."),
        demo("sidebar", "Sidebar", WidgetCategory.Navigation, "stdlib/aura/widgets/navigation/Sidebar.at", SupportTier.Composite, "Persistent side navigation.", "NavigationRail / drawer-like surface", "Tablet behavior is the main reference here."),
        demo("menubar", "MenuBar", WidgetCategory.Navigation, "stdlib/aura/widgets/navigation/MenuBar.at", SupportTier.Composite, "Desktop-like command bar.", "Top row + anchored menus", "Android has no exact first-party menubar."),
        demo("dropdownmenu", "DropdownMenu", WidgetCategory.Navigation, "stdlib/aura/widgets/navigation/DropdownMenu.at", SupportTier.Native, "Anchored action menu.", "DropdownMenu + DropdownMenuItem", "A straightforward overlay mapping."),
        demo("navlink", "NavLink", WidgetCategory.Navigation, "stdlib/aura/widgets/navigation/NavLink.at", SupportTier.Composite, "Route-aware link element.", "Clickable text/button + selected route state", "Route awareness is the interesting part for a2jet."),

        demo("dialog", "Dialog", WidgetCategory.Overlay, "stdlib/aura/widgets/overlay/Dialog.at", SupportTier.Native, "Modal confirmation or content dialog.", "AlertDialog / Dialog", "State-driven presentation is key."),
        demo("alertdialog", "AlertDialog", WidgetCategory.Overlay, "stdlib/aura/widgets/overlay/AlertDialog.at", SupportTier.Native, "High-importance destructive confirmation.", "AlertDialog", "Maps closely to Material patterns."),
        demo("sheet", "Sheet", WidgetCategory.Overlay, "stdlib/aura/widgets/overlay/Sheet.at", SupportTier.Composite, "Bottom sheet surface.", "ModalBottomSheet", "Important mobile-native target for a2jet."),
        demo("drawer", "Drawer", WidgetCategory.Overlay, "stdlib/aura/widgets/overlay/Drawer.at", SupportTier.Composite, "Slide-out navigation panel.", "ModalNavigationDrawer", "Mainly a shell/pattern mapping."),
        demo("popover", "Popover", WidgetCategory.Overlay, "stdlib/aura/widgets/overlay/Popover.at", SupportTier.Composite, "Anchored contextual surface.", "DropdownMenu or Popup", "Android usually expresses this as a menu-like overlay."),
        demo("tooltip", "Tooltip", WidgetCategory.Overlay, "stdlib/aura/widgets/overlay/Tooltip.at", SupportTier.Composite, "Small contextual helper text.", "TooltipBox / custom popup", "Touch UI often changes the trigger semantics."),
        demo("hovercard", "HoverCard", WidgetCategory.Overlay, "stdlib/aura/widgets/overlay/HoverCard.at", SupportTier.Composite, "Peek preview surface.", "Custom popup card on press", "No true hover on phone, so the pattern is adapted."),
        demo("contextmenu", "ContextMenu", WidgetCategory.Overlay, "stdlib/aura/widgets/overlay/ContextMenu.at", SupportTier.Composite, "Secondary action menu.", "IconButton + DropdownMenu", "Long-press or overflow affordance is typical on Android."),

        demo("alert", "Alert", WidgetCategory.Feedback, "stdlib/aura/widgets/feedback/Alert.at", SupportTier.Composite, "Inline status message.", "Card + icon + text", "No single Material3 alert block exists."),
        demo("toast", "Toast", WidgetCategory.Feedback, "stdlib/aura/widgets/feedback/Toast.at", SupportTier.Composite, "Transient system feedback.", "Android Toast", "Useful for direct platform feedback mapping."),
        demo("progress", "Progress", WidgetCategory.Feedback, "stdlib/aura/widgets/feedback/Progress.at", SupportTier.Native, "Loading and completion progress.", "LinearProgressIndicator / CircularProgressIndicator", "Determinate and indeterminate states should both be shown."),
        demo("sonner", "Sonner", WidgetCategory.Feedback, "stdlib/aura/widgets/feedback/Sonner.at", SupportTier.Composite, "Queued toast/snackbar notifications.", "SnackbarHostState + Scaffold host", "A more app-integrated feedback pattern than Toast."),

        demo("table", "Table", WidgetCategory.Data, "stdlib/aura/widgets/data/Table.at", SupportTier.Composite, "Structured rows and columns.", "Column + rows + dividers", "Compose has no built-in table."),
        demo("datatable", "DataTable", WidgetCategory.Data, "stdlib/aura/widgets/data/DataTable.at", SupportTier.Composite, "Tabular data with filtering and sorting hints.", "Custom table + chip filters + sort state", "A2jet will need a higher-level composite strategy."),
        demo("calendar", "Calendar", WidgetCategory.Data, "stdlib/aura/widgets/data/Calendar.at", SupportTier.Composite, "Date selection surface.", "DatePicker", "Closest first-party target in Material3."),
        demo("grid", "Grid", WidgetCategory.Data, "stdlib/aura/widgets/data/Grid.at", SupportTier.Native, "Multi-column lazy layout.", "LazyVerticalGrid", "Grid cells and spacing map cleanly."),
        demo("griditem", "GridItem", WidgetCategory.Data, "stdlib/aura/widgets/data/GridItem.at", SupportTier.Native, "Single grid cell item.", "Card or Box inside LazyVerticalGrid", "Mostly a structural child pattern."),
        demo("list", "List", WidgetCategory.Data, "stdlib/aura/widgets/data/List.at", SupportTier.Native, "Lazy vertical list container.", "LazyColumn", "One of the most important generator targets."),
        demo("listitem", "ListItem", WidgetCategory.Data, "stdlib/aura/widgets/data/ListItem.at", SupportTier.Native, "Structured list row.", "ListItem", "Supports headline/supporting content slots."),
    )

    val byId = demos.associateBy { it.id }

    fun require(id: String): WidgetDemo = requireNotNull(byId[id]) { "Unknown widget demo id: $id" }

    fun firstForSection(section: AppSection): WidgetDemo? = demos.firstOrNull { it.section == section }

    fun groupedForSection(section: AppSection): Map<WidgetCategory, List<WidgetDemo>> =
        demos.filter { it.section == section }.groupBy { it.category }
}

