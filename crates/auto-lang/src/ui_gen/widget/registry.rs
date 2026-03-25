//! Widget Registry
//!
//! This module provides the registry for looking up widget specifications.
//! The registry stores widget specs and allows case-insensitive lookup by tag name.

use super::spec::{BackendMapping, WidgetCategory, WidgetSpec};
use std::collections::HashMap;

/// Core widgets that are auto-imported (no explicit `use` statement needed)
pub const AUTO_IMPORTED_WIDGETS: &[&str] = &[
    // Layout
    "col", "row", "stack", "scroll", "center",
    // Display
    "text", "image",
    // Form
    "button", "input",
    // Feedback
    "alert", "progress",
];

/// Widget registry for looking up widget specifications
pub struct WidgetRegistry {
    widgets: HashMap<String, WidgetSpec>,
}

impl WidgetRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
        }
    }

    /// Create registry with default widgets
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }

    /// Register default widget specifications
    fn register_defaults(&mut self) {
        self.register_layout_widgets();
        self.register_form_widgets();
        self.register_display_widgets();
        self.register_overlay_widgets();
        self.register_navigation_widgets();
        self.register_feedback_widgets();
        self.register_data_widgets();
        self.register_semantic_widgets();
    }

    fn register_layout_widgets(&mut self) {
        // Column
        let mut col = WidgetSpec::new("Column", WidgetCategory::Layout)
            .with_alias("col");
        col.has_children = true;
        col.backends.insert("ark".to_string(), BackendMapping {
            component: "Column".to_string(),
            import: None, // Built-in
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        col.backends.insert("jet".to_string(), BackendMapping {
            component: "Column".to_string(),
            import: Some("androidx.compose.foundation.layout.Column".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        col.backends.insert("vue".to_string(), BackendMapping {
            component: "div".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(col);

        // Center - Column with center alignment (syntax sugar)
        let mut center = WidgetSpec::new("Center", WidgetCategory::Layout)
            .with_alias("center");
        center.has_children = true;
        center.default_props.insert("style".to_string(), "w-full h-full".to_string());
        center.default_props.insert("align".to_string(), "center".to_string());
        center.default_props.insert("arrange".to_string(), "center".to_string());
        center.backends.insert("ark".to_string(), BackendMapping {
            component: "Column".to_string(),
            import: None, // Built-in
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        center.backends.insert("jet".to_string(), BackendMapping {
            component: "Column".to_string(),
            import: Some("androidx.compose.foundation.layout.Column".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        center.backends.insert("vue".to_string(), BackendMapping {
            component: "div".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(center);

        // Row
        let mut row = WidgetSpec::new("Row", WidgetCategory::Layout)
            .with_alias("row");
        row.has_children = true;
        row.backends.insert("ark".to_string(), BackendMapping {
            component: "Row".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        row.backends.insert("jet".to_string(), BackendMapping {
            component: "Row".to_string(),
            import: Some("androidx.compose.foundation.layout.Row".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        row.backends.insert("vue".to_string(), BackendMapping {
            component: "div".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(row);

        // Stack
        let mut stack = WidgetSpec::new("Stack", WidgetCategory::Layout)
            .with_alias("stack")
            .with_alias("box");
        stack.has_children = true;
        stack.backends.insert("ark".to_string(), BackendMapping {
            component: "Stack".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        stack.backends.insert("jet".to_string(), BackendMapping {
            component: "Box".to_string(),
            import: Some("androidx.compose.foundation.layout.Box".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        stack.backends.insert("vue".to_string(), BackendMapping {
            component: "div".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(stack);

        // Scroll
        let mut scroll = WidgetSpec::new("Scroll", WidgetCategory::Layout)
            .with_alias("scroll");
        scroll.has_children = true;
        scroll.backends.insert("ark".to_string(), BackendMapping {
            component: "Scroll".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(scroll);

        // Spacer - fills available space in Row/Column (maps to Blank in ArkTS, Spacer in Compose)
        let mut spacer = WidgetSpec::new("Spacer", WidgetCategory::Layout)
            .with_alias("spacer");
        spacer.has_children = false;
        spacer.backends.insert("ark".to_string(), BackendMapping {
            component: "Blank".to_string(),
            import: None, // Built-in
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        spacer.backends.insert("jet".to_string(), BackendMapping {
            component: "Spacer".to_string(),
            import: Some("androidx.compose.foundation.layout.Spacer".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        spacer.backends.insert("vue".to_string(), BackendMapping {
            component: "div".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(spacer);

        // Card
        let mut card = WidgetSpec::new("Card", WidgetCategory::Layout)
            .with_alias("card");
        card.has_children = true;
        card.backends.insert("ark".to_string(), BackendMapping {
            component: "Card".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        card.backends.insert("jet".to_string(), BackendMapping {
            component: "Card".to_string(),
            import: Some("androidx.compose.material3.Card".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        card.backends.insert("vue".to_string(), BackendMapping {
            component: "Card".to_string(),
            import: Some("@/components/ui/card".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(card);

        // CardHeader
        let mut card_header = WidgetSpec::new("CardHeader", WidgetCategory::Layout)
            .with_alias("card-header");
        card_header.has_children = true;
        card_header.backends.insert("vue".to_string(), BackendMapping {
            component: "CardHeader".to_string(),
            import: Some("@/components/ui/card".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(card_header);

        // CardContent
        let mut card_content = WidgetSpec::new("CardContent", WidgetCategory::Layout)
            .with_alias("card-content");
        card_content.has_children = true;
        card_content.backends.insert("vue".to_string(), BackendMapping {
            component: "CardContent".to_string(),
            import: Some("@/components/ui/card".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(card_content);

        // CardFooter
        let mut card_footer = WidgetSpec::new("CardFooter", WidgetCategory::Layout)
            .with_alias("card-footer");
        card_footer.has_children = true;
        card_footer.backends.insert("vue".to_string(), BackendMapping {
            component: "CardFooter".to_string(),
            import: Some("@/components/ui/card".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(card_footer);

        // CardTitle
        let mut card_title = WidgetSpec::new("CardTitle", WidgetCategory::Layout)
            .with_alias("card-title");
        card_title.has_children = true;
        card_title.backends.insert("vue".to_string(), BackendMapping {
            component: "CardTitle".to_string(),
            import: Some("@/components/ui/card".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(card_title);

        // CardDescription
        let mut card_description = WidgetSpec::new("CardDescription", WidgetCategory::Layout)
            .with_alias("card-description");
        card_description.has_children = true;
        card_description.backends.insert("vue".to_string(), BackendMapping {
            component: "CardDescription".to_string(),
            import: Some("@/components/ui/card".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(card_description);

        // ScrollArea
        let mut scroll_area = WidgetSpec::new("ScrollArea", WidgetCategory::Layout)
            .with_alias("scroll-area");
        scroll_area.has_children = true;
        scroll_area.backends.insert("ark".to_string(), BackendMapping {
            component: "Scroll".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        scroll_area.backends.insert("vue".to_string(), BackendMapping {
            component: "ScrollArea".to_string(),
            import: Some("@/components/ui/scroll-area".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(scroll_area);

        // ScrollAreaViewport
        let mut scroll_viewport = WidgetSpec::new("ScrollAreaViewport", WidgetCategory::Layout)
            .with_alias("scroll-area-viewport");
        scroll_viewport.has_children = true;
        scroll_viewport.backends.insert("vue".to_string(), BackendMapping {
            component: "ScrollAreaViewport".to_string(),
            import: Some("@/components/ui/scroll-area".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(scroll_viewport);

        // ScrollAreaScrollbar
        let mut scroll_scrollbar = WidgetSpec::new("ScrollAreaScrollbar", WidgetCategory::Layout)
            .with_alias("scroll-area-scrollbar");
        scroll_scrollbar.backends.insert("vue".to_string(), BackendMapping {
            component: "ScrollAreaScrollbar".to_string(),
            import: Some("@/components/ui/scroll-area".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(scroll_scrollbar);

        // ScrollAreaThumb
        let mut scroll_thumb = WidgetSpec::new("ScrollAreaThumb", WidgetCategory::Layout)
            .with_alias("scroll-area-thumb");
        scroll_thumb.backends.insert("vue".to_string(), BackendMapping {
            component: "ScrollAreaThumb".to_string(),
            import: Some("@/components/ui/scroll-area".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(scroll_thumb);

        // AspectRatio
        let mut aspect_ratio = WidgetSpec::new("AspectRatio", WidgetCategory::Layout)
            .with_alias("aspect-ratio");
        aspect_ratio.has_children = true;
        aspect_ratio.backends.insert("ark".to_string(), BackendMapping {
            component: "AspectRatio".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        aspect_ratio.backends.insert("vue".to_string(), BackendMapping {
            component: "AspectRatio".to_string(),
            import: Some("@/components/ui/aspect-ratio".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(aspect_ratio);

        // Collapsible
        let mut collapsible = WidgetSpec::new("Collapsible", WidgetCategory::Layout)
            .with_alias("collapsible");
        collapsible.has_children = true;
        collapsible.backends.insert("vue".to_string(), BackendMapping {
            component: "Collapsible".to_string(),
            import: Some("@/components/ui/collapsible".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(collapsible);

        // CollapsibleTrigger
        let mut collapsible_trigger = WidgetSpec::new("CollapsibleTrigger", WidgetCategory::Layout)
            .with_alias("collapsible-trigger");
        collapsible_trigger.has_children = true;
        collapsible_trigger.backends.insert("vue".to_string(), BackendMapping {
            component: "CollapsibleTrigger".to_string(),
            import: Some("@/components/ui/collapsible".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(collapsible_trigger);

        // CollapsibleContent
        let mut collapsible_content = WidgetSpec::new("CollapsibleContent", WidgetCategory::Layout)
            .with_alias("collapsible-content");
        collapsible_content.has_children = true;
        collapsible_content.backends.insert("vue".to_string(), BackendMapping {
            component: "CollapsibleContent".to_string(),
            import: Some("@/components/ui/collapsible".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(collapsible_content);

        // Accordion
        let mut accordion = WidgetSpec::new("Accordion", WidgetCategory::Layout)
            .with_alias("accordion");
        accordion.has_children = true;
        accordion.backends.insert("ark".to_string(), BackendMapping {
            component: "Accordion".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        accordion.backends.insert("vue".to_string(), BackendMapping {
            component: "Accordion".to_string(),
            import: Some("@/components/ui/accordion".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(accordion);

        // AccordionItem
        let mut accordion_item = WidgetSpec::new("AccordionItem", WidgetCategory::Layout)
            .with_alias("accordion-item");
        accordion_item.has_children = true;
        accordion_item.backends.insert("vue".to_string(), BackendMapping {
            component: "AccordionItem".to_string(),
            import: Some("@/components/ui/accordion".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(accordion_item);

        // AccordionTrigger
        let mut accordion_trigger = WidgetSpec::new("AccordionTrigger", WidgetCategory::Layout)
            .with_alias("accordion-trigger");
        accordion_trigger.has_children = true;
        accordion_trigger.backends.insert("vue".to_string(), BackendMapping {
            component: "AccordionTrigger".to_string(),
            import: Some("@/components/ui/accordion".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(accordion_trigger);

        // AccordionContent
        let mut accordion_content = WidgetSpec::new("AccordionContent", WidgetCategory::Layout)
            .with_alias("accordion-content");
        accordion_content.has_children = true;
        accordion_content.backends.insert("vue".to_string(), BackendMapping {
            component: "AccordionContent".to_string(),
            import: Some("@/components/ui/accordion".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(accordion_content);
    }

    fn register_form_widgets(&mut self) {
        // Button
        let mut button = WidgetSpec::new("Button", WidgetCategory::Form)
            .with_alias("button");
        button.primary_prop = Some("text".to_string());
        button.backends.insert("ark".to_string(), BackendMapping {
            component: "Button".to_string(),
            import: Some("@kit.ArkUI".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        button.backends.insert("jet".to_string(), BackendMapping {
            component: "Button".to_string(),
            import: Some("androidx.compose.material3.Button".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        button.backends.insert("vue".to_string(), BackendMapping {
            component: "Button".to_string(),
            import: Some("@/components/ui/button".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(button);

        // Input (TextInput in Ark)
        let mut input = WidgetSpec::new("Input", WidgetCategory::Form)
            .with_alias("input");
        input.backends.insert("ark".to_string(), BackendMapping {
            component: "TextInput".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        input.backends.insert("jet".to_string(), BackendMapping {
            component: "OutlinedTextField".to_string(),
            import: Some("androidx.compose.material3.OutlinedTextField".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        input.backends.insert("vue".to_string(), BackendMapping {
            component: "Input".to_string(),
            import: Some("@/components/ui/input".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(input);

        // Checkbox
        let mut checkbox = WidgetSpec::new("Checkbox", WidgetCategory::Form)
            .with_alias("checkbox");
        checkbox.backends.insert("ark".to_string(), BackendMapping {
            component: "Checkbox".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        checkbox.backends.insert("jet".to_string(), BackendMapping {
            component: "Checkbox".to_string(),
            import: Some("androidx.compose.material3.Checkbox".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        checkbox.backends.insert("vue".to_string(), BackendMapping {
            component: "Checkbox".to_string(),
            import: Some("@/components/ui/checkbox".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(checkbox);

        // Switch
        let mut switch = WidgetSpec::new("Switch", WidgetCategory::Form)
            .with_alias("switch")
            .with_alias("toggle");
        switch.backends.insert("ark".to_string(), BackendMapping {
            component: "Toggle".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        switch.backends.insert("jet".to_string(), BackendMapping {
            component: "Switch".to_string(),
            import: Some("androidx.compose.material3.Switch".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        switch.backends.insert("vue".to_string(), BackendMapping {
            component: "Switch".to_string(),
            import: Some("@/components/ui/switch".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(switch);

        // Select
        let mut select = WidgetSpec::new("Select", WidgetCategory::Form)
            .with_alias("select");
        select.has_children = true;
        select.backends.insert("ark".to_string(), BackendMapping {
            component: "Select".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        select.backends.insert("vue".to_string(), BackendMapping {
            component: "Select".to_string(),
            import: Some("@/components/ui/select".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(select);

        // SelectTrigger
        let mut select_trigger = WidgetSpec::new("SelectTrigger", WidgetCategory::Form)
            .with_alias("select-trigger");
        select_trigger.has_children = true;
        select_trigger.backends.insert("vue".to_string(), BackendMapping {
            component: "SelectTrigger".to_string(),
            import: Some("@/components/ui/select".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(select_trigger);

        // SelectValue
        let mut select_value = WidgetSpec::new("SelectValue", WidgetCategory::Form)
            .with_alias("select-value");
        select_value.backends.insert("vue".to_string(), BackendMapping {
            component: "SelectValue".to_string(),
            import: Some("@/components/ui/select".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(select_value);

        // SelectContent
        let mut select_content = WidgetSpec::new("SelectContent", WidgetCategory::Form)
            .with_alias("select-content");
        select_content.has_children = true;
        select_content.backends.insert("vue".to_string(), BackendMapping {
            component: "SelectContent".to_string(),
            import: Some("@/components/ui/select".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(select_content);

        // SelectItem
        let mut select_item = WidgetSpec::new("SelectItem", WidgetCategory::Form)
            .with_alias("select-item");
        select_item.has_children = true;
        select_item.primary_prop = Some("value".to_string());
        select_item.backends.insert("vue".to_string(), BackendMapping {
            component: "SelectItem".to_string(),
            import: Some("@/components/ui/select".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(select_item);

        // SelectGroup
        let mut select_group = WidgetSpec::new("SelectGroup", WidgetCategory::Form)
            .with_alias("select-group");
        select_group.has_children = true;
        select_group.backends.insert("vue".to_string(), BackendMapping {
            component: "SelectGroup".to_string(),
            import: Some("@/components/ui/select".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(select_group);

        // SelectLabel
        let mut select_label = WidgetSpec::new("SelectLabel", WidgetCategory::Form)
            .with_alias("select-label");
        select_label.has_children = true;
        select_label.backends.insert("vue".to_string(), BackendMapping {
            component: "SelectLabel".to_string(),
            import: Some("@/components/ui/select".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(select_label);

        // Slider
        let mut slider = WidgetSpec::new("Slider", WidgetCategory::Form)
            .with_alias("slider");
        slider.backends.insert("ark".to_string(), BackendMapping {
            component: "Slider".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        slider.backends.insert("jet".to_string(), BackendMapping {
            component: "Slider".to_string(),
            import: Some("androidx.compose.material3.Slider".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        slider.backends.insert("vue".to_string(), BackendMapping {
            component: "Slider".to_string(),
            import: Some("@/components/ui/slider".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(slider);

        // RadioGroup
        let mut radio_group = WidgetSpec::new("RadioGroup", WidgetCategory::Form)
            .with_alias("radio-group");
        radio_group.has_children = true;
        radio_group.backends.insert("ark".to_string(), BackendMapping {
            component: "RadioGroup".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        radio_group.backends.insert("vue".to_string(), BackendMapping {
            component: "RadioGroup".to_string(),
            import: Some("@/components/ui/radio-group".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(radio_group);

        // RadioItem
        let mut radio_item = WidgetSpec::new("RadioItem", WidgetCategory::Form)
            .with_alias("radio-item");
        radio_item.has_children = true;
        radio_item.backends.insert("vue".to_string(), BackendMapping {
            component: "RadioGroupItem".to_string(),
            import: Some("@/components/ui/radio-group".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(radio_item);

        // Textarea
        let mut textarea = WidgetSpec::new("Textarea", WidgetCategory::Form)
            .with_alias("textarea");
        textarea.backends.insert("ark".to_string(), BackendMapping {
            component: "TextArea".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        textarea.backends.insert("jet".to_string(), BackendMapping {
            component: "OutlinedTextField".to_string(),
            import: Some("androidx.compose.material3.OutlinedTextField".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        textarea.backends.insert("vue".to_string(), BackendMapping {
            component: "Textarea".to_string(),
            import: Some("@/components/ui/textarea".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(textarea);

        // Form
        let mut form = WidgetSpec::new("Form", WidgetCategory::Form)
            .with_alias("form");
        form.has_children = true;
        form.backends.insert("vue".to_string(), BackendMapping {
            component: "form".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(form);

        // FormField
        let mut form_field = WidgetSpec::new("FormField", WidgetCategory::Form)
            .with_alias("form-field");
        form_field.has_children = true;
        form_field.backends.insert("vue".to_string(), BackendMapping {
            component: "FormField".to_string(),
            import: Some("@/components/ui/form".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(form_field);

        // FormLabel
        let mut form_label = WidgetSpec::new("FormLabel", WidgetCategory::Form)
            .with_alias("form-label");
        form_label.has_children = true;
        form_label.backends.insert("vue".to_string(), BackendMapping {
            component: "FormLabel".to_string(),
            import: Some("@/components/ui/form".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(form_label);

        // FormControl
        let mut form_control = WidgetSpec::new("FormControl", WidgetCategory::Form)
            .with_alias("form-control");
        form_control.has_children = true;
        form_control.backends.insert("vue".to_string(), BackendMapping {
            component: "FormControl".to_string(),
            import: Some("@/components/ui/form".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(form_control);

        // FormDescription
        let mut form_description = WidgetSpec::new("FormDescription", WidgetCategory::Form)
            .with_alias("form-description");
        form_description.has_children = true;
        form_description.backends.insert("vue".to_string(), BackendMapping {
            component: "FormDescription".to_string(),
            import: Some("@/components/ui/form".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(form_description);

        // FormMessage
        let mut form_message = WidgetSpec::new("FormMessage", WidgetCategory::Form)
            .with_alias("form-message");
        form_message.has_children = true;
        form_message.backends.insert("vue".to_string(), BackendMapping {
            component: "FormMessage".to_string(),
            import: Some("@/components/ui/form".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(form_message);
    }

    fn register_display_widgets(&mut self) {
        // Text
        let mut text = WidgetSpec::new("Text", WidgetCategory::Display)
            .with_alias("text");
        text.primary_prop = Some("text".to_string());
        text.backends.insert("ark".to_string(), BackendMapping {
            component: "Text".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        text.backends.insert("jet".to_string(), BackendMapping {
            component: "Text".to_string(),
            import: Some("androidx.compose.material3.Text".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        text.backends.insert("vue".to_string(), BackendMapping {
            component: "span".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(text);

        // Image
        let mut image = WidgetSpec::new("Image", WidgetCategory::Display)
            .with_alias("image");
        image.primary_prop = Some("src".to_string());
        image.backends.insert("ark".to_string(), BackendMapping {
            component: "Image".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        image.backends.insert("jet".to_string(), BackendMapping {
            component: "Image".to_string(),
            import: Some("androidx.compose.foundation.Image".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        image.backends.insert("vue".to_string(), BackendMapping {
            component: "img".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(image);

        // Badge
        let mut badge = WidgetSpec::new("Badge", WidgetCategory::Display)
            .with_alias("badge");
        badge.has_children = true;
        badge.backends.insert("ark".to_string(), BackendMapping {
            component: "Badge".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        badge.backends.insert("vue".to_string(), BackendMapping {
            component: "Badge".to_string(),
            import: Some("@/components/ui/badge".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(badge);

        // Avatar
        let mut avatar = WidgetSpec::new("Avatar", WidgetCategory::Display)
            .with_alias("avatar");
        avatar.has_children = true;
        avatar.backends.insert("ark".to_string(), BackendMapping {
            component: "Avatar".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        avatar.backends.insert("vue".to_string(), BackendMapping {
            component: "Avatar".to_string(),
            import: Some("@/components/ui/avatar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(avatar);

        // AvatarImage
        let mut avatar_image = WidgetSpec::new("AvatarImage", WidgetCategory::Display)
            .with_alias("avatar-image");
        avatar_image.primary_prop = Some("src".to_string());
        avatar_image.backends.insert("vue".to_string(), BackendMapping {
            component: "AvatarImage".to_string(),
            import: Some("@/components/ui/avatar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(avatar_image);

        // AvatarFallback
        let mut avatar_fallback = WidgetSpec::new("AvatarFallback", WidgetCategory::Display)
            .with_alias("avatar-fallback");
        avatar_fallback.has_children = true;
        avatar_fallback.backends.insert("vue".to_string(), BackendMapping {
            component: "AvatarFallback".to_string(),
            import: Some("@/components/ui/avatar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(avatar_fallback);

        // Separator
        let mut separator = WidgetSpec::new("Separator", WidgetCategory::Display)
            .with_alias("separator");
        separator.backends.insert("ark".to_string(), BackendMapping {
            component: "Divider".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        separator.backends.insert("jet".to_string(), BackendMapping {
            component: "HorizontalDivider".to_string(),
            import: Some("androidx.compose.material3.HorizontalDivider".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        separator.backends.insert("vue".to_string(), BackendMapping {
            component: "Separator".to_string(),
            import: Some("@/components/ui/separator".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(separator);

        // Skeleton
        let mut skeleton = WidgetSpec::new("Skeleton", WidgetCategory::Display)
            .with_alias("skeleton");
        skeleton.backends.insert("ark".to_string(), BackendMapping {
            component: "Skeleton".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        skeleton.backends.insert("vue".to_string(), BackendMapping {
            component: "Skeleton".to_string(),
            import: Some("@/components/ui/skeleton".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(skeleton);
    }

    fn register_navigation_widgets(&mut self) {
        // Swiper
        let mut swiper = WidgetSpec::new("Swiper", WidgetCategory::Navigation)
            .with_alias("swiper");
        swiper.has_children = true;
        swiper.backends.insert("ark".to_string(), BackendMapping {
            component: "Swiper".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(swiper);

        // Tabs
        let mut tabs = WidgetSpec::new("Tabs", WidgetCategory::Navigation)
            .with_alias("tabs");
        tabs.has_children = true;
        tabs.backends.insert("ark".to_string(), BackendMapping {
            component: "Tabs".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        tabs.backends.insert("vue".to_string(), BackendMapping {
            component: "Tabs".to_string(),
            import: Some("@/components/ui/tabs".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(tabs);

        // TabsList
        let mut tabs_list = WidgetSpec::new("TabsList", WidgetCategory::Navigation)
            .with_alias("tabs-list");
        tabs_list.has_children = true;
        tabs_list.backends.insert("ark".to_string(), BackendMapping {
            component: "Row".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        tabs_list.backends.insert("vue".to_string(), BackendMapping {
            component: "TabsList".to_string(),
            import: Some("@/components/ui/tabs".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(tabs_list);

        // TabsTrigger
        let mut tabs_trigger = WidgetSpec::new("TabsTrigger", WidgetCategory::Navigation)
            .with_alias("tabs-trigger");
        tabs_trigger.has_children = true;
        tabs_trigger.primary_prop = Some("id".to_string());
        tabs_trigger.backends.insert("ark".to_string(), BackendMapping {
            component: "Text".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        tabs_trigger.backends.insert("vue".to_string(), BackendMapping {
            component: "TabsTrigger".to_string(),
            import: Some("@/components/ui/tabs".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(tabs_trigger);

        // TabsContent
        let mut tabs_content = WidgetSpec::new("TabsContent", WidgetCategory::Navigation)
            .with_alias("tabs-content");
        tabs_content.has_children = true;
        tabs_content.primary_prop = Some("id".to_string());
        tabs_content.backends.insert("ark".to_string(), BackendMapping {
            component: "TabContent".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        tabs_content.backends.insert("vue".to_string(), BackendMapping {
            component: "TabsContent".to_string(),
            import: Some("@/components/ui/tabs".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(tabs_content);

        // Navigation - root navigation container for HarmonyOS
        let mut navigation = WidgetSpec::new("Navigation", WidgetCategory::Navigation)
            .with_alias("navigation");
        navigation.has_children = true;
        navigation.primary_prop = Some("pathStack".to_string());
        navigation.backends.insert("ark".to_string(), BackendMapping {
            component: "Navigation".to_string(),
            import: None, // Built-in
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(navigation);

        // NavDestination - for detail pages in navigation stack
        let mut nav_destination = WidgetSpec::new("NavDestination", WidgetCategory::Navigation)
            .with_alias("nav-destination");
        nav_destination.has_children = true;
        nav_destination.backends.insert("ark".to_string(), BackendMapping {
            component: "NavDestination".to_string(),
            import: None, // Built-in
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(nav_destination);

        // Breadcrumb
        let mut breadcrumb = WidgetSpec::new("Breadcrumb", WidgetCategory::Navigation)
            .with_alias("breadcrumb");
        breadcrumb.has_children = true;
        breadcrumb.backends.insert("vue".to_string(), BackendMapping {
            component: "Breadcrumb".to_string(),
            import: Some("@/components/ui/breadcrumb".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(breadcrumb);

        // BreadcrumbList
        let mut breadcrumb_list = WidgetSpec::new("BreadcrumbList", WidgetCategory::Navigation)
            .with_alias("breadcrumb-list");
        breadcrumb_list.has_children = true;
        breadcrumb_list.backends.insert("vue".to_string(), BackendMapping {
            component: "BreadcrumbList".to_string(),
            import: Some("@/components/ui/breadcrumb".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(breadcrumb_list);

        // BreadcrumbItem
        let mut breadcrumb_item = WidgetSpec::new("BreadcrumbItem", WidgetCategory::Navigation)
            .with_alias("breadcrumb-item");
        breadcrumb_item.has_children = true;
        breadcrumb_item.backends.insert("vue".to_string(), BackendMapping {
            component: "BreadcrumbItem".to_string(),
            import: Some("@/components/ui/breadcrumb".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(breadcrumb_item);

        // BreadcrumbLink
        let mut breadcrumb_link = WidgetSpec::new("BreadcrumbLink", WidgetCategory::Navigation)
            .with_alias("breadcrumb-link");
        breadcrumb_link.has_children = true;
        breadcrumb_link.backends.insert("vue".to_string(), BackendMapping {
            component: "BreadcrumbLink".to_string(),
            import: Some("@/components/ui/breadcrumb".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(breadcrumb_link);

        // BreadcrumbPage
        let mut breadcrumb_page = WidgetSpec::new("BreadcrumbPage", WidgetCategory::Navigation)
            .with_alias("breadcrumb-page");
        breadcrumb_page.has_children = true;
        breadcrumb_page.backends.insert("vue".to_string(), BackendMapping {
            component: "BreadcrumbPage".to_string(),
            import: Some("@/components/ui/breadcrumb".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(breadcrumb_page);

        // BreadcrumbSeparator
        let mut breadcrumb_sep = WidgetSpec::new("BreadcrumbSeparator", WidgetCategory::Navigation)
            .with_alias("breadcrumb-separator");
        breadcrumb_sep.backends.insert("vue".to_string(), BackendMapping {
            component: "BreadcrumbSeparator".to_string(),
            import: Some("@/components/ui/breadcrumb".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(breadcrumb_sep);

        // NavigationMenu
        let mut nav_menu = WidgetSpec::new("NavigationMenu", WidgetCategory::Navigation)
            .with_alias("navigation-menu")
            .with_alias("nav-menu");
        nav_menu.has_children = true;
        nav_menu.backends.insert("vue".to_string(), BackendMapping {
            component: "NavigationMenu".to_string(),
            import: Some("@/components/ui/navigation-menu".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(nav_menu);

        // NavigationMenuList
        let mut nav_menu_list = WidgetSpec::new("NavigationMenuList", WidgetCategory::Navigation)
            .with_alias("navigation-menu-list")
            .with_alias("nav-menu-list");
        nav_menu_list.has_children = true;
        nav_menu_list.backends.insert("vue".to_string(), BackendMapping {
            component: "NavigationMenuList".to_string(),
            import: Some("@/components/ui/navigation-menu".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(nav_menu_list);

        // NavigationMenuItem
        let mut nav_menu_item = WidgetSpec::new("NavigationMenuItem", WidgetCategory::Navigation)
            .with_alias("navigation-menu-item")
            .with_alias("nav-menu-item");
        nav_menu_item.has_children = true;
        nav_menu_item.backends.insert("vue".to_string(), BackendMapping {
            component: "NavigationMenuItem".to_string(),
            import: Some("@/components/ui/navigation-menu".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(nav_menu_item);

        // NavigationMenuTrigger
        let mut nav_menu_trigger = WidgetSpec::new("NavigationMenuTrigger", WidgetCategory::Navigation)
            .with_alias("navigation-menu-trigger")
            .with_alias("nav-menu-trigger");
        nav_menu_trigger.has_children = true;
        nav_menu_trigger.backends.insert("vue".to_string(), BackendMapping {
            component: "NavigationMenuTrigger".to_string(),
            import: Some("@/components/ui/navigation-menu".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(nav_menu_trigger);

        // NavigationMenuContent
        let mut nav_menu_content = WidgetSpec::new("NavigationMenuContent", WidgetCategory::Navigation)
            .with_alias("navigation-menu-content")
            .with_alias("nav-menu-content");
        nav_menu_content.has_children = true;
        nav_menu_content.backends.insert("vue".to_string(), BackendMapping {
            component: "NavigationMenuContent".to_string(),
            import: Some("@/components/ui/navigation-menu".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(nav_menu_content);

        // NavigationMenuLink
        let mut nav_menu_link = WidgetSpec::new("NavigationMenuLink", WidgetCategory::Navigation)
            .with_alias("navigation-menu-link")
            .with_alias("nav-menu-link");
        nav_menu_link.has_children = true;
        nav_menu_link.backends.insert("vue".to_string(), BackendMapping {
            component: "NavigationMenuLink".to_string(),
            import: Some("@/components/ui/navigation-menu".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(nav_menu_link);

        // Pagination
        let mut pagination = WidgetSpec::new("Pagination", WidgetCategory::Navigation)
            .with_alias("pagination");
        pagination.has_children = true;
        pagination.backends.insert("vue".to_string(), BackendMapping {
            component: "Pagination".to_string(),
            import: Some("@/components/ui/pagination".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(pagination);

        // PaginationList
        let mut pagination_list = WidgetSpec::new("PaginationList", WidgetCategory::Navigation)
            .with_alias("pagination-list");
        pagination_list.has_children = true;
        pagination_list.backends.insert("vue".to_string(), BackendMapping {
            component: "PaginationList".to_string(),
            import: Some("@/components/ui/pagination".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(pagination_list);

        // PaginationItem
        let mut pagination_item = WidgetSpec::new("PaginationItem", WidgetCategory::Navigation)
            .with_alias("pagination-item");
        pagination_item.has_children = true;
        pagination_item.backends.insert("vue".to_string(), BackendMapping {
            component: "PaginationItem".to_string(),
            import: Some("@/components/ui/pagination".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(pagination_item);

        // PaginationLink
        let mut pagination_link = WidgetSpec::new("PaginationLink", WidgetCategory::Navigation)
            .with_alias("pagination-link");
        pagination_link.backends.insert("vue".to_string(), BackendMapping {
            component: "PaginationLink".to_string(),
            import: Some("@/components/ui/pagination".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(pagination_link);

        // PaginationEllipsis
        let mut pagination_ellipsis = WidgetSpec::new("PaginationEllipsis", WidgetCategory::Navigation)
            .with_alias("pagination-ellipsis");
        pagination_ellipsis.backends.insert("vue".to_string(), BackendMapping {
            component: "PaginationEllipsis".to_string(),
            import: Some("@/components/ui/pagination".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(pagination_ellipsis);

        // PaginationNext
        let mut pagination_next = WidgetSpec::new("PaginationNext", WidgetCategory::Navigation)
            .with_alias("pagination-next");
        pagination_next.backends.insert("vue".to_string(), BackendMapping {
            component: "PaginationNext".to_string(),
            import: Some("@/components/ui/pagination".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(pagination_next);

        // PaginationPrevious
        let mut pagination_prev = WidgetSpec::new("PaginationPrevious", WidgetCategory::Navigation)
            .with_alias("pagination-previous");
        pagination_prev.backends.insert("vue".to_string(), BackendMapping {
            component: "PaginationPrevious".to_string(),
            import: Some("@/components/ui/pagination".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(pagination_prev);

        // Sidebar
        let mut sidebar = WidgetSpec::new("Sidebar", WidgetCategory::Navigation)
            .with_alias("sidebar");
        sidebar.has_children = true;
        sidebar.backends.insert("vue".to_string(), BackendMapping {
            component: "Sidebar".to_string(),
            import: Some("@/components/ui/sidebar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(sidebar);

        // SidebarHeader
        let mut sidebar_header = WidgetSpec::new("SidebarHeader", WidgetCategory::Navigation)
            .with_alias("sidebar-header");
        sidebar_header.has_children = true;
        sidebar_header.backends.insert("vue".to_string(), BackendMapping {
            component: "SidebarHeader".to_string(),
            import: Some("@/components/ui/sidebar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(sidebar_header);

        // SidebarContent
        let mut sidebar_content = WidgetSpec::new("SidebarContent", WidgetCategory::Navigation)
            .with_alias("sidebar-content");
        sidebar_content.has_children = true;
        sidebar_content.backends.insert("vue".to_string(), BackendMapping {
            component: "SidebarContent".to_string(),
            import: Some("@/components/ui/sidebar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(sidebar_content);

        // SidebarFooter
        let mut sidebar_footer = WidgetSpec::new("SidebarFooter", WidgetCategory::Navigation)
            .with_alias("sidebar-footer");
        sidebar_footer.has_children = true;
        sidebar_footer.backends.insert("vue".to_string(), BackendMapping {
            component: "SidebarFooter".to_string(),
            import: Some("@/components/ui/sidebar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(sidebar_footer);

        // SidebarMenu
        let mut sidebar_menu = WidgetSpec::new("SidebarMenu", WidgetCategory::Navigation)
            .with_alias("sidebar-menu");
        sidebar_menu.has_children = true;
        sidebar_menu.backends.insert("vue".to_string(), BackendMapping {
            component: "SidebarMenu".to_string(),
            import: Some("@/components/ui/sidebar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(sidebar_menu);

        // SidebarMenuItem
        let mut sidebar_menu_item = WidgetSpec::new("SidebarMenuItem", WidgetCategory::Navigation)
            .with_alias("sidebar-menu-item");
        sidebar_menu_item.has_children = true;
        sidebar_menu_item.backends.insert("vue".to_string(), BackendMapping {
            component: "SidebarMenuItem".to_string(),
            import: Some("@/components/ui/sidebar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(sidebar_menu_item);

        // SidebarMenuButton
        let mut sidebar_menu_btn = WidgetSpec::new("SidebarMenuButton", WidgetCategory::Navigation)
            .with_alias("sidebar-menu-button");
        sidebar_menu_btn.has_children = true;
        sidebar_menu_btn.backends.insert("vue".to_string(), BackendMapping {
            component: "SidebarMenuButton".to_string(),
            import: Some("@/components/ui/sidebar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(sidebar_menu_btn);

        // MenuBar
        let mut menu_bar = WidgetSpec::new("MenuBar", WidgetCategory::Navigation)
            .with_alias("menu-bar");
        menu_bar.has_children = true;
        menu_bar.backends.insert("vue".to_string(), BackendMapping {
            component: "Menubar".to_string(),
            import: Some("@/components/ui/menubar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(menu_bar);

        // MenuBarMenu
        let mut menu_bar_menu = WidgetSpec::new("MenuBarMenu", WidgetCategory::Navigation)
            .with_alias("menu-bar-menu");
        menu_bar_menu.has_children = true;
        menu_bar_menu.backends.insert("vue".to_string(), BackendMapping {
            component: "MenubarMenu".to_string(),
            import: Some("@/components/ui/menubar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(menu_bar_menu);

        // MenuBarTrigger
        let mut menu_bar_trigger = WidgetSpec::new("MenuBarTrigger", WidgetCategory::Navigation)
            .with_alias("menu-bar-trigger");
        menu_bar_trigger.has_children = true;
        menu_bar_trigger.backends.insert("vue".to_string(), BackendMapping {
            component: "MenubarTrigger".to_string(),
            import: Some("@/components/ui/menubar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(menu_bar_trigger);

        // MenuBarContent
        let mut menu_bar_content = WidgetSpec::new("MenuBarContent", WidgetCategory::Navigation)
            .with_alias("menu-bar-content");
        menu_bar_content.has_children = true;
        menu_bar_content.backends.insert("vue".to_string(), BackendMapping {
            component: "MenubarContent".to_string(),
            import: Some("@/components/ui/menubar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(menu_bar_content);

        // MenuBarItem
        let mut menu_bar_item = WidgetSpec::new("MenuBarItem", WidgetCategory::Navigation)
            .with_alias("menu-bar-item");
        menu_bar_item.has_children = true;
        menu_bar_item.backends.insert("vue".to_string(), BackendMapping {
            component: "MenubarItem".to_string(),
            import: Some("@/components/ui/menubar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(menu_bar_item);

        // DropdownMenu
        let mut dropdown_menu = WidgetSpec::new("DropdownMenu", WidgetCategory::Navigation)
            .with_alias("dropdown-menu");
        dropdown_menu.has_children = true;
        dropdown_menu.backends.insert("vue".to_string(), BackendMapping {
            component: "DropdownMenu".to_string(),
            import: Some("@/components/ui/dropdown-menu".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(dropdown_menu);

        // DropdownMenuTrigger
        let mut dropdown_trigger = WidgetSpec::new("DropdownMenuTrigger", WidgetCategory::Navigation)
            .with_alias("dropdown-menu-trigger");
        dropdown_trigger.has_children = true;
        dropdown_trigger.backends.insert("vue".to_string(), BackendMapping {
            component: "DropdownMenuTrigger".to_string(),
            import: Some("@/components/ui/dropdown-menu".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(dropdown_trigger);

        // DropdownMenuContent
        let mut dropdown_content = WidgetSpec::new("DropdownMenuContent", WidgetCategory::Navigation)
            .with_alias("dropdown-menu-content");
        dropdown_content.has_children = true;
        dropdown_content.backends.insert("vue".to_string(), BackendMapping {
            component: "DropdownMenuContent".to_string(),
            import: Some("@/components/ui/dropdown-menu".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(dropdown_content);

        // DropdownMenuItem
        let mut dropdown_item = WidgetSpec::new("DropdownMenuItem", WidgetCategory::Navigation)
            .with_alias("dropdown-menu-item");
        dropdown_item.has_children = true;
        dropdown_item.backends.insert("vue".to_string(), BackendMapping {
            component: "DropdownMenuItem".to_string(),
            import: Some("@/components/ui/dropdown-menu".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(dropdown_item);

        // NavLink
        let mut nav_link = WidgetSpec::new("NavLink", WidgetCategory::Navigation)
            .with_alias("nav-link");
        nav_link.has_children = true;
        nav_link.backends.insert("vue".to_string(), BackendMapping {
            component: "NavLink".to_string(),
            import: Some("@/components/ui/nav-link".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(nav_link);
    }

    fn register_overlay_widgets(&mut self) {
        // Dialog
        let mut dialog = WidgetSpec::new("Dialog", WidgetCategory::Overlay)
            .with_alias("dialog");
        dialog.has_children = true;
        dialog.backends.insert("ark".to_string(), BackendMapping {
            component: "AlertDialog".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        dialog.backends.insert("vue".to_string(), BackendMapping {
            component: "Dialog".to_string(),
            import: Some("@/components/ui/dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(dialog);

        // DialogTrigger
        let mut dialog_trigger = WidgetSpec::new("DialogTrigger", WidgetCategory::Overlay)
            .with_alias("dialog-trigger");
        dialog_trigger.has_children = true;
        dialog_trigger.backends.insert("ark".to_string(), BackendMapping {
            component: "Button".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        dialog_trigger.backends.insert("vue".to_string(), BackendMapping {
            component: "DialogTrigger".to_string(),
            import: Some("@/components/ui/dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(dialog_trigger);

        // DialogContent
        let mut dialog_content = WidgetSpec::new("DialogContent", WidgetCategory::Overlay)
            .with_alias("dialog-content");
        dialog_content.has_children = true;
        dialog_content.backends.insert("ark".to_string(), BackendMapping {
            component: "Column".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        dialog_content.backends.insert("vue".to_string(), BackendMapping {
            component: "DialogContent".to_string(),
            import: Some("@/components/ui/dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(dialog_content);

        // DialogHeader
        let mut dialog_header = WidgetSpec::new("DialogHeader", WidgetCategory::Overlay)
            .with_alias("dialog-header");
        dialog_header.has_children = true;
        dialog_header.backends.insert("ark".to_string(), BackendMapping {
            component: "Column".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        dialog_header.backends.insert("vue".to_string(), BackendMapping {
            component: "DialogHeader".to_string(),
            import: Some("@/components/ui/dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(dialog_header);

        // DialogFooter
        let mut dialog_footer = WidgetSpec::new("DialogFooter", WidgetCategory::Overlay)
            .with_alias("dialog-footer");
        dialog_footer.has_children = true;
        dialog_footer.backends.insert("ark".to_string(), BackendMapping {
            component: "Row".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        dialog_footer.backends.insert("vue".to_string(), BackendMapping {
            component: "DialogFooter".to_string(),
            import: Some("@/components/ui/dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(dialog_footer);

        // DialogTitle
        let mut dialog_title = WidgetSpec::new("DialogTitle", WidgetCategory::Overlay)
            .with_alias("dialog-title");
        dialog_title.has_children = true;
        dialog_title.backends.insert("ark".to_string(), BackendMapping {
            component: "Text".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        dialog_title.backends.insert("vue".to_string(), BackendMapping {
            component: "DialogTitle".to_string(),
            import: Some("@/components/ui/dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(dialog_title);

        // DialogDescription
        let mut dialog_desc = WidgetSpec::new("DialogDescription", WidgetCategory::Overlay)
            .with_alias("dialog-description");
        dialog_desc.has_children = true;
        dialog_desc.backends.insert("ark".to_string(), BackendMapping {
            component: "Text".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        dialog_desc.backends.insert("vue".to_string(), BackendMapping {
            component: "DialogDescription".to_string(),
            import: Some("@/components/ui/dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(dialog_desc);

        // AlertDialog
        let mut alert_dialog = WidgetSpec::new("AlertDialog", WidgetCategory::Overlay)
            .with_alias("alert-dialog");
        alert_dialog.has_children = true;
        alert_dialog.backends.insert("ark".to_string(), BackendMapping {
            component: "AlertDialog".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        alert_dialog.backends.insert("vue".to_string(), BackendMapping {
            component: "AlertDialog".to_string(),
            import: Some("@/components/ui/alert-dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(alert_dialog);

        // AlertDialogTrigger
        let mut alert_dialog_trigger = WidgetSpec::new("AlertDialogTrigger", WidgetCategory::Overlay)
            .with_alias("alert-dialog-trigger");
        alert_dialog_trigger.has_children = true;
        alert_dialog_trigger.backends.insert("vue".to_string(), BackendMapping {
            component: "AlertDialogTrigger".to_string(),
            import: Some("@/components/ui/alert-dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(alert_dialog_trigger);

        // AlertDialogContent
        let mut alert_dialog_content = WidgetSpec::new("AlertDialogContent", WidgetCategory::Overlay)
            .with_alias("alert-dialog-content");
        alert_dialog_content.has_children = true;
        alert_dialog_content.backends.insert("vue".to_string(), BackendMapping {
            component: "AlertDialogContent".to_string(),
            import: Some("@/components/ui/alert-dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(alert_dialog_content);

        // AlertDialogHeader
        let mut alert_dialog_header = WidgetSpec::new("AlertDialogHeader", WidgetCategory::Overlay)
            .with_alias("alert-dialog-header");
        alert_dialog_header.has_children = true;
        alert_dialog_header.backends.insert("vue".to_string(), BackendMapping {
            component: "AlertDialogHeader".to_string(),
            import: Some("@/components/ui/alert-dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(alert_dialog_header);

        // AlertDialogFooter
        let mut alert_dialog_footer = WidgetSpec::new("AlertDialogFooter", WidgetCategory::Overlay)
            .with_alias("alert-dialog-footer");
        alert_dialog_footer.has_children = true;
        alert_dialog_footer.backends.insert("vue".to_string(), BackendMapping {
            component: "AlertDialogFooter".to_string(),
            import: Some("@/components/ui/alert-dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(alert_dialog_footer);

        // AlertDialogTitle
        let mut alert_dialog_title = WidgetSpec::new("AlertDialogTitle", WidgetCategory::Overlay)
            .with_alias("alert-dialog-title");
        alert_dialog_title.has_children = true;
        alert_dialog_title.backends.insert("vue".to_string(), BackendMapping {
            component: "AlertDialogTitle".to_string(),
            import: Some("@/components/ui/alert-dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(alert_dialog_title);

        // AlertDialogDescription
        let mut alert_dialog_desc = WidgetSpec::new("AlertDialogDescription", WidgetCategory::Overlay)
            .with_alias("alert-dialog-description");
        alert_dialog_desc.has_children = true;
        alert_dialog_desc.backends.insert("vue".to_string(), BackendMapping {
            component: "AlertDialogDescription".to_string(),
            import: Some("@/components/ui/alert-dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(alert_dialog_desc);

        // AlertDialogAction
        let mut alert_dialog_action = WidgetSpec::new("AlertDialogAction", WidgetCategory::Overlay)
            .with_alias("alert-dialog-action");
        alert_dialog_action.has_children = true;
        alert_dialog_action.backends.insert("vue".to_string(), BackendMapping {
            component: "AlertDialogAction".to_string(),
            import: Some("@/components/ui/alert-dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(alert_dialog_action);

        // AlertDialogCancel
        let mut alert_dialog_cancel = WidgetSpec::new("AlertDialogCancel", WidgetCategory::Overlay)
            .with_alias("alert-dialog-cancel");
        alert_dialog_cancel.has_children = true;
        alert_dialog_cancel.backends.insert("vue".to_string(), BackendMapping {
            component: "AlertDialogCancel".to_string(),
            import: Some("@/components/ui/alert-dialog".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(alert_dialog_cancel);

        // Sheet
        let mut sheet = WidgetSpec::new("Sheet", WidgetCategory::Overlay)
            .with_alias("sheet");
        sheet.has_children = true;
        sheet.backends.insert("vue".to_string(), BackendMapping {
            component: "Sheet".to_string(),
            import: Some("@/components/ui/sheet".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(sheet);

        // SheetTrigger
        let mut sheet_trigger = WidgetSpec::new("SheetTrigger", WidgetCategory::Overlay)
            .with_alias("sheet-trigger");
        sheet_trigger.has_children = true;
        sheet_trigger.backends.insert("vue".to_string(), BackendMapping {
            component: "SheetTrigger".to_string(),
            import: Some("@/components/ui/sheet".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(sheet_trigger);

        // SheetContent
        let mut sheet_content = WidgetSpec::new("SheetContent", WidgetCategory::Overlay)
            .with_alias("sheet-content");
        sheet_content.has_children = true;
        sheet_content.backends.insert("vue".to_string(), BackendMapping {
            component: "SheetContent".to_string(),
            import: Some("@/components/ui/sheet".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(sheet_content);

        // SheetHeader
        let mut sheet_header = WidgetSpec::new("SheetHeader", WidgetCategory::Overlay)
            .with_alias("sheet-header");
        sheet_header.has_children = true;
        sheet_header.backends.insert("vue".to_string(), BackendMapping {
            component: "SheetHeader".to_string(),
            import: Some("@/components/ui/sheet".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(sheet_header);

        // SheetFooter
        let mut sheet_footer = WidgetSpec::new("SheetFooter", WidgetCategory::Overlay)
            .with_alias("sheet-footer");
        sheet_footer.has_children = true;
        sheet_footer.backends.insert("vue".to_string(), BackendMapping {
            component: "SheetFooter".to_string(),
            import: Some("@/components/ui/sheet".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(sheet_footer);

        // SheetTitle
        let mut sheet_title = WidgetSpec::new("SheetTitle", WidgetCategory::Overlay)
            .with_alias("sheet-title");
        sheet_title.has_children = true;
        sheet_title.backends.insert("vue".to_string(), BackendMapping {
            component: "SheetTitle".to_string(),
            import: Some("@/components/ui/sheet".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(sheet_title);

        // SheetDescription
        let mut sheet_desc = WidgetSpec::new("SheetDescription", WidgetCategory::Overlay)
            .with_alias("sheet-description");
        sheet_desc.has_children = true;
        sheet_desc.backends.insert("vue".to_string(), BackendMapping {
            component: "SheetDescription".to_string(),
            import: Some("@/components/ui/sheet".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(sheet_desc);

        // Drawer
        let mut drawer = WidgetSpec::new("Drawer", WidgetCategory::Overlay)
            .with_alias("drawer");
        drawer.has_children = true;
        drawer.backends.insert("vue".to_string(), BackendMapping {
            component: "Drawer".to_string(),
            import: Some("@/components/ui/drawer".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(drawer);

        // DrawerTrigger
        let mut drawer_trigger = WidgetSpec::new("DrawerTrigger", WidgetCategory::Overlay)
            .with_alias("drawer-trigger");
        drawer_trigger.has_children = true;
        drawer_trigger.backends.insert("vue".to_string(), BackendMapping {
            component: "DrawerTrigger".to_string(),
            import: Some("@/components/ui/drawer".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(drawer_trigger);

        // DrawerContent
        let mut drawer_content = WidgetSpec::new("DrawerContent", WidgetCategory::Overlay)
            .with_alias("drawer-content");
        drawer_content.has_children = true;
        drawer_content.backends.insert("vue".to_string(), BackendMapping {
            component: "DrawerContent".to_string(),
            import: Some("@/components/ui/drawer".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(drawer_content);

        // DrawerHeader
        let mut drawer_header = WidgetSpec::new("DrawerHeader", WidgetCategory::Overlay)
            .with_alias("drawer-header");
        drawer_header.has_children = true;
        drawer_header.backends.insert("vue".to_string(), BackendMapping {
            component: "DrawerHeader".to_string(),
            import: Some("@/components/ui/drawer".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(drawer_header);

        // DrawerFooter
        let mut drawer_footer = WidgetSpec::new("DrawerFooter", WidgetCategory::Overlay)
            .with_alias("drawer-footer");
        drawer_footer.has_children = true;
        drawer_footer.backends.insert("vue".to_string(), BackendMapping {
            component: "DrawerFooter".to_string(),
            import: Some("@/components/ui/drawer".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(drawer_footer);

        // DrawerTitle
        let mut drawer_title = WidgetSpec::new("DrawerTitle", WidgetCategory::Overlay)
            .with_alias("drawer-title");
        drawer_title.has_children = true;
        drawer_title.backends.insert("vue".to_string(), BackendMapping {
            component: "DrawerTitle".to_string(),
            import: Some("@/components/ui/drawer".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(drawer_title);

        // DrawerDescription
        let mut drawer_desc = WidgetSpec::new("DrawerDescription", WidgetCategory::Overlay)
            .with_alias("drawer-description");
        drawer_desc.has_children = true;
        drawer_desc.backends.insert("vue".to_string(), BackendMapping {
            component: "DrawerDescription".to_string(),
            import: Some("@/components/ui/drawer".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(drawer_desc);

        // Popover
        let mut popover = WidgetSpec::new("Popover", WidgetCategory::Overlay)
            .with_alias("popover");
        popover.has_children = true;
        popover.backends.insert("vue".to_string(), BackendMapping {
            component: "Popover".to_string(),
            import: Some("@/components/ui/popover".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(popover);

        // PopoverTrigger
        let mut popover_trigger = WidgetSpec::new("PopoverTrigger", WidgetCategory::Overlay)
            .with_alias("popover-trigger");
        popover_trigger.has_children = true;
        popover_trigger.backends.insert("vue".to_string(), BackendMapping {
            component: "PopoverTrigger".to_string(),
            import: Some("@/components/ui/popover".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(popover_trigger);

        // PopoverContent
        let mut popover_content = WidgetSpec::new("PopoverContent", WidgetCategory::Overlay)
            .with_alias("popover-content");
        popover_content.has_children = true;
        popover_content.backends.insert("vue".to_string(), BackendMapping {
            component: "PopoverContent".to_string(),
            import: Some("@/components/ui/popover".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(popover_content);

        // Tooltip
        let mut tooltip = WidgetSpec::new("Tooltip", WidgetCategory::Overlay)
            .with_alias("tooltip");
        tooltip.has_children = true;
        tooltip.backends.insert("vue".to_string(), BackendMapping {
            component: "Tooltip".to_string(),
            import: Some("@/components/ui/tooltip".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(tooltip);

        // TooltipTrigger
        let mut tooltip_trigger = WidgetSpec::new("TooltipTrigger", WidgetCategory::Overlay)
            .with_alias("tooltip-trigger");
        tooltip_trigger.has_children = true;
        tooltip_trigger.backends.insert("vue".to_string(), BackendMapping {
            component: "TooltipTrigger".to_string(),
            import: Some("@/components/ui/tooltip".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(tooltip_trigger);

        // TooltipContent
        let mut tooltip_content = WidgetSpec::new("TooltipContent", WidgetCategory::Overlay)
            .with_alias("tooltip-content");
        tooltip_content.has_children = true;
        tooltip_content.backends.insert("vue".to_string(), BackendMapping {
            component: "TooltipContent".to_string(),
            import: Some("@/components/ui/tooltip".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(tooltip_content);

        // HoverCard
        let mut hover_card = WidgetSpec::new("HoverCard", WidgetCategory::Overlay)
            .with_alias("hover-card");
        hover_card.has_children = true;
        hover_card.backends.insert("vue".to_string(), BackendMapping {
            component: "HoverCard".to_string(),
            import: Some("@/components/ui/hover-card".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(hover_card);

        // HoverCardTrigger
        let mut hover_card_trigger = WidgetSpec::new("HoverCardTrigger", WidgetCategory::Overlay)
            .with_alias("hover-card-trigger");
        hover_card_trigger.has_children = true;
        hover_card_trigger.backends.insert("vue".to_string(), BackendMapping {
            component: "HoverCardTrigger".to_string(),
            import: Some("@/components/ui/hover-card".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(hover_card_trigger);

        // HoverCardContent
        let mut hover_card_content = WidgetSpec::new("HoverCardContent", WidgetCategory::Overlay)
            .with_alias("hover-card-content");
        hover_card_content.has_children = true;
        hover_card_content.backends.insert("vue".to_string(), BackendMapping {
            component: "HoverCardContent".to_string(),
            import: Some("@/components/ui/hover-card".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(hover_card_content);

        // ContextMenu
        let mut context_menu = WidgetSpec::new("ContextMenu", WidgetCategory::Overlay)
            .with_alias("context-menu");
        context_menu.has_children = true;
        context_menu.backends.insert("vue".to_string(), BackendMapping {
            component: "ContextMenu".to_string(),
            import: Some("@/components/ui/context-menu".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(context_menu);

        // ContextMenuTrigger
        let mut context_menu_trigger = WidgetSpec::new("ContextMenuTrigger", WidgetCategory::Overlay)
            .with_alias("context-menu-trigger");
        context_menu_trigger.has_children = true;
        context_menu_trigger.backends.insert("vue".to_string(), BackendMapping {
            component: "ContextMenuTrigger".to_string(),
            import: Some("@/components/ui/context-menu".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(context_menu_trigger);

        // ContextMenuContent
        let mut context_menu_content = WidgetSpec::new("ContextMenuContent", WidgetCategory::Overlay)
            .with_alias("context-menu-content");
        context_menu_content.has_children = true;
        context_menu_content.backends.insert("vue".to_string(), BackendMapping {
            component: "ContextMenuContent".to_string(),
            import: Some("@/components/ui/context-menu".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(context_menu_content);

        // ContextMenuItem
        let mut context_menu_item = WidgetSpec::new("ContextMenuItem", WidgetCategory::Overlay)
            .with_alias("context-menu-item");
        context_menu_item.has_children = true;
        context_menu_item.backends.insert("vue".to_string(), BackendMapping {
            component: "ContextMenuItem".to_string(),
            import: Some("@/components/ui/context-menu".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(context_menu_item);
    }

    fn register_feedback_widgets(&mut self) {
        // Alert
        let mut alert = WidgetSpec::new("Alert", WidgetCategory::Feedback)
            .with_alias("alert");
        alert.has_children = true;
        alert.backends.insert("ark".to_string(), BackendMapping {
            component: "Alert".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        alert.backends.insert("vue".to_string(), BackendMapping {
            component: "Alert".to_string(),
            import: Some("@/components/ui/alert".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(alert);

        // AlertTitle
        let mut alert_title = WidgetSpec::new("AlertTitle", WidgetCategory::Feedback)
            .with_alias("alert-title");
        alert_title.has_children = true;
        alert_title.backends.insert("vue".to_string(), BackendMapping {
            component: "AlertTitle".to_string(),
            import: Some("@/components/ui/alert".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(alert_title);

        // AlertDescription
        let mut alert_desc = WidgetSpec::new("AlertDescription", WidgetCategory::Feedback)
            .with_alias("alert-description");
        alert_desc.has_children = true;
        alert_desc.backends.insert("vue".to_string(), BackendMapping {
            component: "AlertDescription".to_string(),
            import: Some("@/components/ui/alert".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(alert_desc);

        // Toast
        let mut toast = WidgetSpec::new("Toast", WidgetCategory::Feedback)
            .with_alias("toast");
        toast.has_children = true;
        toast.backends.insert("ark".to_string(), BackendMapping {
            component: "Toast".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        toast.backends.insert("vue".to_string(), BackendMapping {
            component: "Toast".to_string(),
            import: Some("@/components/ui/toast".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(toast);

        // ToastProvider
        let mut toast_provider = WidgetSpec::new("ToastProvider", WidgetCategory::Feedback)
            .with_alias("toast-provider");
        toast_provider.has_children = true;
        toast_provider.backends.insert("vue".to_string(), BackendMapping {
            component: "ToastProvider".to_string(),
            import: Some("@/components/ui/toast".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(toast_provider);

        // ToastViewport
        let mut toast_viewport = WidgetSpec::new("ToastViewport", WidgetCategory::Feedback)
            .with_alias("toast-viewport");
        toast_viewport.backends.insert("vue".to_string(), BackendMapping {
            component: "ToastViewport".to_string(),
            import: Some("@/components/ui/toast".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(toast_viewport);

        // ToastAction
        let mut toast_action = WidgetSpec::new("ToastAction", WidgetCategory::Feedback)
            .with_alias("toast-action");
        toast_action.has_children = true;
        toast_action.backends.insert("vue".to_string(), BackendMapping {
            component: "ToastAction".to_string(),
            import: Some("@/components/ui/toast".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(toast_action);

        // ToastClose
        let mut toast_close = WidgetSpec::new("ToastClose", WidgetCategory::Feedback)
            .with_alias("toast-close");
        toast_close.backends.insert("vue".to_string(), BackendMapping {
            component: "ToastClose".to_string(),
            import: Some("@/components/ui/toast".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(toast_close);

        // ToastTitle
        let mut toast_title = WidgetSpec::new("ToastTitle", WidgetCategory::Feedback)
            .with_alias("toast-title");
        toast_title.has_children = true;
        toast_title.backends.insert("vue".to_string(), BackendMapping {
            component: "ToastTitle".to_string(),
            import: Some("@/components/ui/toast".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(toast_title);

        // ToastDescription
        let mut toast_desc = WidgetSpec::new("ToastDescription", WidgetCategory::Feedback)
            .with_alias("toast-description");
        toast_desc.has_children = true;
        toast_desc.backends.insert("vue".to_string(), BackendMapping {
            component: "ToastDescription".to_string(),
            import: Some("@/components/ui/toast".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(toast_desc);

        // Progress
        let mut progress = WidgetSpec::new("Progress", WidgetCategory::Feedback)
            .with_alias("progress");
        progress.backends.insert("ark".to_string(), BackendMapping {
            component: "Progress".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        progress.backends.insert("jet".to_string(), BackendMapping {
            component: "LinearProgressIndicator".to_string(),
            import: Some("androidx.compose.material3.LinearProgressIndicator".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        progress.backends.insert("vue".to_string(), BackendMapping {
            component: "Progress".to_string(),
            import: Some("@/components/ui/progress".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(progress);

        // Sonner
        let mut sonner = WidgetSpec::new("Sonner", WidgetCategory::Feedback)
            .with_alias("sonner");
        sonner.backends.insert("vue".to_string(), BackendMapping {
            component: "Sonner".to_string(),
            import: Some("@/components/ui/sonner".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(sonner);
    }

    fn register_data_widgets(&mut self) {
        // Table
        let mut table = WidgetSpec::new("Table", WidgetCategory::Data)
            .with_alias("table");
        table.has_children = true;
        table.backends.insert("ark".to_string(), BackendMapping {
            component: "Table".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        table.backends.insert("vue".to_string(), BackendMapping {
            component: "Table".to_string(),
            import: Some("@/components/ui/table".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(table);

        // TableHeader
        let mut table_header = WidgetSpec::new("TableHeader", WidgetCategory::Data)
            .with_alias("table-header");
        table_header.has_children = true;
        table_header.backends.insert("ark".to_string(), BackendMapping {
            component: "Column".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        table_header.backends.insert("vue".to_string(), BackendMapping {
            component: "TableHeader".to_string(),
            import: Some("@/components/ui/table".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(table_header);

        // TableBody
        let mut table_body = WidgetSpec::new("TableBody", WidgetCategory::Data)
            .with_alias("table-body");
        table_body.has_children = true;
        table_body.backends.insert("ark".to_string(), BackendMapping {
            component: "Column".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        table_body.backends.insert("vue".to_string(), BackendMapping {
            component: "TableBody".to_string(),
            import: Some("@/components/ui/table".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(table_body);

        // TableFooter
        let mut table_footer = WidgetSpec::new("TableFooter", WidgetCategory::Data)
            .with_alias("table-footer");
        table_footer.has_children = true;
        table_footer.backends.insert("ark".to_string(), BackendMapping {
            component: "Column".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        table_footer.backends.insert("vue".to_string(), BackendMapping {
            component: "TableFooter".to_string(),
            import: Some("@/components/ui/table".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(table_footer);

        // TableRow
        let mut table_row = WidgetSpec::new("TableRow", WidgetCategory::Data)
            .with_alias("table-row");
        table_row.has_children = true;
        table_row.backends.insert("ark".to_string(), BackendMapping {
            component: "Row".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        table_row.backends.insert("vue".to_string(), BackendMapping {
            component: "TableRow".to_string(),
            import: Some("@/components/ui/table".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(table_row);

        // TableHead
        let mut table_head = WidgetSpec::new("TableHead", WidgetCategory::Data)
            .with_alias("table-head");
        table_head.has_children = true;
        table_head.backends.insert("ark".to_string(), BackendMapping {
            component: "Text".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        table_head.backends.insert("vue".to_string(), BackendMapping {
            component: "TableHead".to_string(),
            import: Some("@/components/ui/table".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(table_head);

        // TableCell
        let mut table_cell = WidgetSpec::new("TableCell", WidgetCategory::Data)
            .with_alias("table-cell");
        table_cell.has_children = true;
        table_cell.backends.insert("ark".to_string(), BackendMapping {
            component: "Text".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        table_cell.backends.insert("vue".to_string(), BackendMapping {
            component: "TableCell".to_string(),
            import: Some("@/components/ui/table".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(table_cell);

        // TableCaption
        let mut table_caption = WidgetSpec::new("TableCaption", WidgetCategory::Data)
            .with_alias("table-caption");
        table_caption.has_children = true;
        table_caption.backends.insert("ark".to_string(), BackendMapping {
            component: "Text".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        table_caption.backends.insert("vue".to_string(), BackendMapping {
            component: "TableCaption".to_string(),
            import: Some("@/components/ui/table".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(table_caption);

        // DataTable
        let mut data_table = WidgetSpec::new("DataTable", WidgetCategory::Data)
            .with_alias("data-table");
        data_table.has_children = true;
        data_table.backends.insert("vue".to_string(), BackendMapping {
            component: "DataTable".to_string(),
            import: Some("@/components/ui/data-table".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(data_table);

        // Calendar
        let mut calendar = WidgetSpec::new("Calendar", WidgetCategory::Data)
            .with_alias("calendar");
        calendar.has_children = true;
        calendar.backends.insert("ark".to_string(), BackendMapping {
            component: "Calendar".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        calendar.backends.insert("jet".to_string(), BackendMapping {
            component: "DatePicker".to_string(),
            import: Some("androidx.compose.material3.DatePicker".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        calendar.backends.insert("vue".to_string(), BackendMapping {
            component: "Calendar".to_string(),
            import: Some("@/components/ui/calendar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(calendar);

        // CalendarGrid
        let mut calendar_grid = WidgetSpec::new("CalendarGrid", WidgetCategory::Data)
            .with_alias("calendar-grid");
        calendar_grid.has_children = true;
        calendar_grid.backends.insert("vue".to_string(), BackendMapping {
            component: "CalendarGrid".to_string(),
            import: Some("@/components/ui/calendar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(calendar_grid);

        // CalendarHeader
        let mut calendar_header = WidgetSpec::new("CalendarHeader", WidgetCategory::Data)
            .with_alias("calendar-header");
        calendar_header.has_children = true;
        calendar_header.backends.insert("vue".to_string(), BackendMapping {
            component: "CalendarHeader".to_string(),
            import: Some("@/components/ui/calendar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(calendar_header);

        // CalendarHeading
        let mut calendar_heading = WidgetSpec::new("CalendarHeading", WidgetCategory::Data)
            .with_alias("calendar-heading");
        calendar_heading.has_children = true;
        calendar_heading.backends.insert("vue".to_string(), BackendMapping {
            component: "CalendarHeading".to_string(),
            import: Some("@/components/ui/calendar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(calendar_heading);

        // CalendarCell
        let mut calendar_cell = WidgetSpec::new("CalendarCell", WidgetCategory::Data)
            .with_alias("calendar-cell");
        calendar_cell.has_children = true;
        calendar_cell.backends.insert("vue".to_string(), BackendMapping {
            component: "CalendarCell".to_string(),
            import: Some("@/components/ui/calendar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(calendar_cell);

        // CalendarDay
        let mut calendar_day = WidgetSpec::new("CalendarDay", WidgetCategory::Data)
            .with_alias("calendar-day");
        calendar_day.backends.insert("vue".to_string(), BackendMapping {
            component: "CalendarDay".to_string(),
            import: Some("@/components/ui/calendar".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(calendar_day);

        // Grid
        let mut grid = WidgetSpec::new("Grid", WidgetCategory::Data)
            .with_alias("grid");
        grid.has_children = true;
        grid.backends.insert("ark".to_string(), BackendMapping {
            component: "Grid".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        grid.backends.insert("jet".to_string(), BackendMapping {
            component: "LazyVerticalGrid".to_string(),
            import: Some("androidx.compose.foundation.lazy.grid.LazyVerticalGrid".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        grid.backends.insert("vue".to_string(), BackendMapping {
            component: "div".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(grid);

        // GridItem
        let mut grid_item = WidgetSpec::new("GridItem", WidgetCategory::Data)
            .with_alias("grid-item");
        grid_item.has_children = true;
        grid_item.backends.insert("ark".to_string(), BackendMapping {
            component: "GridItem".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        grid_item.backends.insert("jet".to_string(), BackendMapping {
            component: "item".to_string(),
            import: Some("androidx.compose.foundation.lazy.grid.GridItem".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        grid_item.backends.insert("vue".to_string(), BackendMapping {
            component: "div".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(grid_item);

        // List
        let mut list = WidgetSpec::new("List", WidgetCategory::Data)
            .with_alias("list");
        list.has_children = true;
        list.backends.insert("ark".to_string(), BackendMapping {
            component: "List".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        list.backends.insert("jet".to_string(), BackendMapping {
            component: "LazyColumn".to_string(),
            import: Some("androidx.compose.foundation.lazy.LazyColumn".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        list.backends.insert("vue".to_string(), BackendMapping {
            component: "ul".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(list);

        // ListItem
        let mut list_item = WidgetSpec::new("ListItem", WidgetCategory::Data)
            .with_alias("list-item");
        list_item.has_children = true;
        list_item.backends.insert("ark".to_string(), BackendMapping {
            component: "ListItem".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        list_item.backends.insert("jet".to_string(), BackendMapping {
            component: "item".to_string(),
            import: Some("androidx.compose.foundation.lazy.LazyListScope.item".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        list_item.backends.insert("vue".to_string(), BackendMapping {
            component: "li".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        });
        self.register(list_item);
    }

    fn register_semantic_widgets(&mut self) {
        // Semantic HTML elements map to Column in Ark
        for tag in ["header", "footer", "nav", "main", "aside", "article", "section"] {
            let mut widget = WidgetSpec::new(tag, WidgetCategory::Semantic);
            widget.has_children = true;
            widget.backends.insert("ark".to_string(), BackendMapping {
                component: "Column".to_string(),
                import: None,
                props: HashMap::new(),
                events: HashMap::new(),
                extra_components: Vec::new(),
            });
            self.register(widget);
        }

        // Heading elements map to Text
        for tag in ["h1", "h2", "h3", "h4", "h5", "h6"] {
            let mut widget = WidgetSpec::new(tag, WidgetCategory::Display);
            widget.primary_prop = Some("text".to_string());
            widget.backends.insert("ark".to_string(), BackendMapping {
                component: "Text".to_string(),
                import: None,
                props: HashMap::new(),
                events: HashMap::new(),
                extra_components: Vec::new(),
            });
            self.register(widget);
        }
    }

    /// Register a widget
    pub fn register(&mut self, spec: WidgetSpec) {
        // Register under the canonical name
        let key = spec.name.to_lowercase();
        let aliases = spec.aliases.clone();
        self.widgets.insert(key.clone(), spec);

        // Register under all aliases (they point to the same spec)
        // Note: We need to clone for each alias
        for alias in aliases {
            if let Some(spec) = self.widgets.get(&key) {
                self.widgets.insert(alias, spec.clone());
            }
        }
    }

    /// Look up a widget by tag name (case-insensitive)
    pub fn get(&self, tag: &str) -> Option<&WidgetSpec> {
        self.widgets.get(&tag.to_lowercase())
    }

    /// Check if a widget exists
    pub fn contains(&self, tag: &str) -> bool {
        self.widgets.contains_key(&tag.to_lowercase())
    }

    /// Check if a widget is auto-imported (no explicit `use` statement needed)
    pub fn is_auto_imported(tag: &str) -> bool {
        AUTO_IMPORTED_WIDGETS.contains(&tag.to_lowercase().as_str())
    }

    /// Get the list of auto-imported widgets
    pub fn auto_imported_widgets() -> &'static [&'static str] {
        AUTO_IMPORTED_WIDGETS
    }

    // ========================================================================
    // Backend-specific helper methods for generators
    // ========================================================================

    /// Get the import path for a widget in a specific backend
    /// Returns None if widget doesn't exist or has no import (native element)
    pub fn get_backend_import(&self, backend: &str, tag: &str) -> Option<String> {
        self.get(tag)
            .and_then(|spec| spec.backend(backend))
            .and_then(|mapping| mapping.import.clone())
    }

    /// Get all component names for a widget in a specific backend
    /// Returns empty vec if widget doesn't exist
    pub fn get_backend_components(&self, backend: &str, tag: &str) -> Vec<String> {
        self.get(tag)
            .and_then(|spec| spec.backend(backend))
            .map(|mapping| mapping.all_components().iter().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    /// Get the primary component name for a widget in a specific backend
    pub fn get_primary_component(&self, backend: &str, tag: &str) -> Option<String> {
        self.get(tag)
            .and_then(|spec| spec.backend(backend))
            .map(|mapping| mapping.primary_component().to_string())
    }

    /// Check if a widget is supported for a specific backend
    pub fn is_backend_supported(&self, backend: &str, tag: &str) -> bool {
        self.get(tag)
            .map(|spec| spec.backend(backend).is_some())
            .unwrap_or(false)
    }

    /// Get all unique imports needed for a set of tags in a specific backend
    /// Returns a map of import_path -> component_names
    pub fn collect_backend_imports(&self, backend: &str, tags: &[&str]) -> HashMap<String, Vec<String>> {
        let mut imports: HashMap<String, Vec<String>> = HashMap::new();

        for tag in tags {
            if let Some(spec) = self.get(tag) {
                if let Some(mapping) = spec.backend(backend) {
                    if let Some(ref import_path) = mapping.import {
                        let components = mapping.all_components();
                        let entry = imports.entry(import_path.clone()).or_default();
                        for comp in components {
                            if !entry.contains(&comp.to_string()) {
                                entry.push(comp.to_string());
                            }
                        }
                    }
                }
            }
        }

        imports
    }

    /// Get property mapping for a widget in a specific backend
    pub fn get_prop_mapping(&self, backend: &str, tag: &str, aura_prop: &str) -> Option<String> {
        self.get(tag)
            .and_then(|spec| spec.backend(backend))
            .and_then(|mapping| mapping.props.get(aura_prop).cloned())
    }

    /// Get event mapping for a widget in a specific backend
    pub fn get_event_mapping(&self, backend: &str, tag: &str, aura_event: &str) -> Option<String> {
        self.get(tag)
            .and_then(|spec| spec.backend(backend))
            .and_then(|mapping| mapping.events.get(aura_event).cloned())
    }

    /// Get default props for a widget
    pub fn get_default_props(&self, tag: &str) -> HashMap<String, String> {
        self.get(tag)
            .map(|spec| spec.default_props.clone())
            .unwrap_or_default()
    }

    /// Iterate over all registered widgets
    pub fn iter(&self) -> impl Iterator<Item = (&String, &WidgetSpec)> {
        self.widgets.iter()
    }

    /// Get all widgets (for generators that need to iterate)
    pub fn all_widgets(&self) -> &HashMap<String, WidgetSpec> {
        &self.widgets
    }
}

impl Default for WidgetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = WidgetRegistry::new();
        assert!(registry.get("button").is_none()); // Empty registry
    }

    #[test]
    fn test_default_widgets_col() {
        let registry = WidgetRegistry::with_defaults();
        let col = registry.get("col").unwrap();
        assert_eq!(col.name, "Column");
        assert_eq!(col.category, WidgetCategory::Layout);
        assert!(col.has_children);
    }

    #[test]
    fn test_default_widgets_button() {
        let registry = WidgetRegistry::with_defaults();
        let button = registry.get("button").unwrap();
        assert_eq!(button.name, "Button");
        assert_eq!(button.category, WidgetCategory::Form);

        let ark_mapping = button.backend("ark").unwrap();
        assert_eq!(ark_mapping.component, "Button");
        assert_eq!(ark_mapping.import, Some("@kit.ArkUI".to_string()));
    }

    #[test]
    fn test_default_widgets_text() {
        let registry = WidgetRegistry::with_defaults();
        let text = registry.get("text").unwrap();
        assert_eq!(text.name, "Text");
        assert_eq!(text.category, WidgetCategory::Display);
    }

    #[test]
    fn test_default_widgets_image() {
        let registry = WidgetRegistry::with_defaults();
        let image = registry.get("image").unwrap();
        assert_eq!(image.name, "Image");

        let ark_mapping = image.backend("ark").unwrap();
        assert_eq!(ark_mapping.component, "Image");
    }

    #[test]
    fn test_semantic_widgets_map_to_column() {
        let registry = WidgetRegistry::with_defaults();
        for tag in ["header", "footer", "nav", "main"] {
            let widget = registry.get(tag).unwrap();
            let ark = widget.backend("ark").unwrap();
            assert_eq!(ark.component, "Column", "{} should map to Column", tag);
        }
    }

    #[test]
    fn test_all_layout_widgets() {
        let registry = WidgetRegistry::with_defaults();
        for tag in ["col", "row", "stack", "scroll", "center"] {
            assert!(registry.contains(tag), "Missing layout widget: {}", tag);
        }
    }

    #[test]
    fn test_center_widget() {
        let registry = WidgetRegistry::with_defaults();
        let center = registry.get("center").unwrap();
        assert_eq!(center.name, "Center");
        assert_eq!(center.category, WidgetCategory::Layout);
        assert!(center.has_children);

        // Check default props
        assert_eq!(center.default_props.get("style"), Some(&"w-full h-full".to_string()));
        assert_eq!(center.default_props.get("align"), Some(&"center".to_string()));
        assert_eq!(center.default_props.get("arrange"), Some(&"center".to_string()));

        // Check backends
        let ark = center.backend("ark").unwrap();
        assert_eq!(ark.component, "Column");
        let jet = center.backend("jet").unwrap();
        assert_eq!(jet.component, "Column");
        let vue = center.backend("vue").unwrap();
        assert_eq!(vue.component, "div");
    }

    #[test]
    fn test_all_form_widgets() {
        let registry = WidgetRegistry::with_defaults();
        for tag in ["button", "input"] {
            assert!(registry.contains(tag), "Missing form widget: {}", tag);
        }
    }

    #[test]
    fn test_case_insensitive_lookup() {
        let registry = WidgetRegistry::with_defaults();
        assert!(registry.get("BUTTON").is_some());
        assert!(registry.get("Button").is_some());
        assert!(registry.get("button").is_some());
    }
}
