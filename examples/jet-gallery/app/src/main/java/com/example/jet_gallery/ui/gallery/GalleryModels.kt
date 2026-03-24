package com.example.jet_gallery.ui.gallery

enum class AppSection(val title: String) {
    Home("Home"),
    Foundation("Foundation"),
    Input("Input"),
    Structure("Structure"),
    Data("Data"),
}

enum class WidgetCategory(val title: String, val section: AppSection) {
    Layout("Layout", AppSection.Foundation),
    Display("Display", AppSection.Foundation),
    Feedback("Feedback", AppSection.Foundation),
    Form("Form", AppSection.Input),
    Navigation("Navigation", AppSection.Structure),
    Overlay("Overlay", AppSection.Structure),
    Data("Data", AppSection.Data),
}

enum class SupportTier(val title: String) {
    Native("Native"),
    Composite("Composite"),
}

data class WidgetDemo(
    val id: String,
    val title: String,
    val category: WidgetCategory,
    val stdlibPath: String,
    val supportTier: SupportTier,
    val description: String,
    val composeTarget: String,
    val notes: List<String> = emptyList(),
) {
    val section: AppSection
        get() = category.section
}

