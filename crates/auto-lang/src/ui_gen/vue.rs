//! Vue3/JavaScript Code Generator
//!
//! Generates Vue 3 Single File Components (SFC) from AURA widgets.
//! Supports two output modes:
//!
//! 1. **Plain Tailwind** - Native HTML elements with Tailwind CSS classes
//! 2. **shadcn-vue** - Pre-built accessible components from shadcn-vue
//!
//! ## Output Format (shadcn-vue mode)
//!
//! ```vue
//! <script setup>
//! import { ref } from 'vue'
//! import { Button } from '@/components/ui/button'
//! import { Input } from '@/components/ui/input'
//!
//! const count = ref(0)
//!
//! const handleInc = () => {
//!   count.value += 1
//! }
//! </script>
//!
//! <template>
//!   <div class="flex flex-col gap-2">
//!     <Button @click="handleInc">Increment</Button>
//!     <Input v-model="count" />
//!   </div>
//! </template>
//! ```
//!
//! ## Output Format (Plain Tailwind mode)
//!
//! ```vue
//! <script setup>
//! import { ref, computed } from 'vue'
//!
//! // State variables → ref()
//! const count = ref(0)
//!
//! // Event handlers
//! const handleInc = () => {
//!   count.value += 1
//! }
//! </script>
//!
//! <template>
//!   <div class="flex flex-col">
//!     <button @click="handleInc">+</button>
//!     <h2>Count: {{ count }}</h2>
//!   </div>
//! </template>
//!
//! <style>
//! /* Component styles */
//! </style>
//! ```
//!
//! ## Supported shadcn-vue Components (Plan 099)
//!
//! ### Content Elements
//! | AURA Tag | shadcn-vue | Props |
//! |----------|------------|-------|
//! | `button` | Button | variant, size, disabled, text→slot |
//! | `input` | Input | v-model, type, placeholder, disabled |
//! | `textarea` | Textarea | v-model, placeholder, rows, disabled |
//! | `checkbox` | Checkbox | v-model:checked, disabled |
//! | `toggle`/`switch` | Switch | v-model:checked, disabled |
//! | `select` | Select | v-model, disabled |
//!
//! ### Layout & Navigation (Phase 3)
//! | AURA Tag | shadcn-vue | Props |
//! |----------|------------|-------|
//! | `scroll` | ScrollArea | class, orientation, hide_delay |
//! | `tabs` | Tabs | v-model, default-value |
//! | `tab` | TabsTrigger | value, disabled, text→slot |
//! | `card` | Card | variant, title→slot |
//! | `divider` | Separator | orientation, decorative, label |
//!
//! ### Overlay & Feedback (Phase 4)
//! | AURA Tag | shadcn-vue | Props |
//! |----------|------------|-------|
//! | `modal` | Dialog | v-model:open, title, description |
//! | `tooltip` | Tooltip | content→slot, side, delay |
//! | `spinner` | Skeleton | class, width, height |
//! | `progress` | Progress | v-model, max |
//! | `badge` | Badge | variant, text→slot |
//!
//! ### Data Components (Phase 5)
//! | AURA Tag | shadcn-vue | Props |
//! |----------|------------|-------|
//! | `table` | Table | class |
//! | `thead`/`tbody`/`tr` | TableHeader/TableBody/TableRow | class |
//! | `th`/`td` | TableHead/TableCell | class, colspan, rowspan |
//! | `tree` | Collapsible | class |
//! | `tree_item` | CollapsibleItem | v-model:open, text→slot |
//! | `avatar` | Avatar | src, name→slot |
//!
//! ### Form Components (Phase 6)
//! | AURA Tag | shadcn-vue | Props |
//! |----------|------------|-------|
//! | `slider` | Slider | v-model, min, max, step, disabled |
//! | `radiogroup` | RadioGroup | v-model, name, disabled |
//! | `radio` | RadioGroupItem | value, id, disabled, label→slot |

use super::{BackendGenerator, GenError, GenResult, WidgetRegistry};
use crate::aura::{AuraEvent, AuraNode, AuraPropValue, AuraStyleBinding, AuraTextContent, AuraWidget, LogicPayload};
use std::collections::{HashMap, HashSet};

// ============================================================================
// shadcn-vue Component Registry (DEPRECATED)
// ============================================================================

/// Maps AURA element tags to shadcn-vue component imports
///
/// **DEPRECATED**: Use `WidgetRegistry` instead. This registry is kept for
/// backward compatibility and will be removed in a future version.
#[deprecated(since = "0.2.0", note = "Use WidgetRegistry instead")]
pub struct ShadcnRegistry {
    /// Component imports needed: tag -> (module_path, component_names)
    components: HashMap<&'static str, (&'static str, Vec<&'static str>)>,
}

#[allow(deprecated)]
impl ShadcnRegistry {
    /// Create registry with all shadcn-vue component mappings
    #[allow(deprecated)]
    pub fn new() -> Self {
        let mut components = HashMap::new();

        // === Content Elements ===
        components.insert("button",
            ("@/components/ui/button", vec!["Button"]));
        components.insert("input",
            ("@/components/ui/input", vec!["Input"]));
        components.insert("textarea",
            ("@/components/ui/textarea", vec!["Textarea"]));
        components.insert("checkbox",
            ("@/components/ui/checkbox", vec!["Checkbox"]));
        components.insert("toggle",
            ("@/components/ui/switch", vec!["Switch"]));
        components.insert("select",
            ("@/components/ui/select", vec!["Select", "SelectContent", "SelectItem", "SelectTrigger", "SelectValue"]));
        components.insert("option",
            ("@/components/ui/select", vec!["SelectItem"]));
        // Select sub-components
        components.insert("selecttrigger",
            ("@/components/ui/select", vec!["SelectTrigger"]));
        components.insert("select-trigger",
            ("@/components/ui/select", vec!["SelectTrigger"]));
        components.insert("selectvalue",
            ("@/components/ui/select", vec!["SelectValue"]));
        components.insert("select-value",
            ("@/components/ui/select", vec!["SelectValue"]));
        components.insert("selectcontent",
            ("@/components/ui/select", vec!["SelectContent"]));
        components.insert("select-content",
            ("@/components/ui/select", vec!["SelectContent"]));
        components.insert("selectitem",
            ("@/components/ui/select", vec!["SelectItem"]));
        components.insert("select-item",
            ("@/components/ui/select", vec!["SelectItem"]));
        components.insert("selectgroup",
            ("@/components/ui/select", vec!["SelectGroup"]));
        components.insert("select-group",
            ("@/components/ui/select", vec!["SelectGroup"]));
        components.insert("selectlabel",
            ("@/components/ui/select", vec!["SelectLabel"]));
        components.insert("select-label",
            ("@/components/ui/select", vec!["SelectLabel"]));
        components.insert("selectseparator",
            ("@/components/ui/select", vec!["SelectSeparator"]));
        components.insert("select-separator",
            ("@/components/ui/select", vec!["SelectSeparator"]));
        components.insert("selectscrollbutton",
            ("@/components/ui/select", vec!["SelectScrollUpButton", "SelectScrollDownButton"]));

        // === Navigation Elements ===
        components.insert("tabs",
            ("@/components/ui/tabs", vec!["Tabs", "TabsList", "TabsTrigger", "TabsContent"]));
        components.insert("tabslist",
            ("@/components/ui/tabs", vec!["TabsList"]));
        components.insert("tabs-list",
            ("@/components/ui/tabs", vec!["TabsList"]));
        components.insert("tabstrigger",
            ("@/components/ui/tabs", vec!["TabsTrigger"]));
        components.insert("tabs-trigger",
            ("@/components/ui/tabs", vec!["TabsTrigger"]));
        components.insert("tabscontent",
            ("@/components/ui/tabs", vec!["TabsContent"]));
        components.insert("tabs-content",
            ("@/components/ui/tabs", vec!["TabsContent"]));
        components.insert("tab",
            ("@/components/ui/tabs", vec!["TabsTrigger", "TabsContent"]));

        // === Overlay Elements ===
        components.insert("modal",
            ("@/components/ui/dialog", vec!["Dialog", "DialogContent", "DialogTrigger", "DialogTitle", "DialogDescription"]));
        components.insert("tooltip",
            ("@/components/ui/tooltip", vec!["Tooltip", "TooltipContent", "TooltipProvider", "TooltipTrigger"]));

        // === Form Elements ===
        components.insert("slider",
            ("@/components/ui/slider", vec!["Slider"]));
        components.insert("radio",
            ("@/components/ui/radio-group", vec!["RadioGroup", "RadioGroupItem"]));
        components.insert("radiogroup",
            ("@/components/ui/radio-group", vec!["RadioGroup"]));
        components.insert("radio-group",
            ("@/components/ui/radio-group", vec!["RadioGroup"]));

        // === Feedback Elements ===
        components.insert("progress",
            ("@/components/ui/progress", vec!["Progress"]));
        components.insert("badge",
            ("@/components/ui/badge", vec!["Badge"]));
        components.insert("spinner",
            ("@/components/ui/skeleton", vec!["Skeleton"]));

        // === Display Elements ===
        components.insert("card",
            ("@/components/ui/card", vec!["Card", "CardHeader", "CardTitle", "CardDescription", "CardContent", "CardFooter"]));
        components.insert("cardheader",
            ("@/components/ui/card", vec!["CardHeader"]));
        components.insert("card-header",
            ("@/components/ui/card", vec!["CardHeader"]));
        components.insert("cardtitle",
            ("@/components/ui/card", vec!["CardTitle"]));
        components.insert("card-title",
            ("@/components/ui/card", vec!["CardTitle"]));
        components.insert("carddescription",
            ("@/components/ui/card", vec!["CardDescription"]));
        components.insert("card-description",
            ("@/components/ui/card", vec!["CardDescription"]));
        components.insert("cardcontent",
            ("@/components/ui/card", vec!["CardContent"]));
        components.insert("card-content",
            ("@/components/ui/card", vec!["CardContent"]));
        components.insert("cardfooter",
            ("@/components/ui/card", vec!["CardFooter"]));
        components.insert("card-footer",
            ("@/components/ui/card", vec!["CardFooter"]));
        components.insert("avatar",
            ("@/components/ui/avatar", vec!["Avatar", "AvatarImage", "AvatarFallback"]));

        // === Display: AspectRatio ===
        components.insert("aspectratio",
            ("@/components/ui/aspect-ratio", vec!["AspectRatio"]));
        components.insert("aspect-ratio",
            ("@/components/ui/aspect-ratio", vec!["AspectRatio"]));

        // === Data Elements ===
        components.insert("table",
            ("@/components/ui/table", vec!["Table", "TableHeader", "TableBody", "TableRow", "TableHead", "TableCell", "TableCaption"]));
        components.insert("thead",
            ("@/components/ui/table", vec!["TableHeader"]));
        components.insert("tbody",
            ("@/components/ui/table", vec!["TableBody"]));
        components.insert("tr",
            ("@/components/ui/table", vec!["TableRow"]));
        components.insert("th",
            ("@/components/ui/table", vec!["TableHead"]));
        components.insert("td",
            ("@/components/ui/table", vec!["TableCell"]));
        components.insert("table_caption",
            ("@/components/ui/table", vec!["TableCaption"]));
        components.insert("table_header",
            ("@/components/ui/table", vec!["TableHeader"]));
        components.insert("table_body",
            ("@/components/ui/table", vec!["TableBody"]));
        components.insert("table_row",
            ("@/components/ui/table", vec!["TableRow"]));
        components.insert("table_head",
            ("@/components/ui/table", vec!["TableHead"]));
        components.insert("table_cell",
            ("@/components/ui/table", vec!["TableCell"]));

        // === Utility Elements ===
        components.insert("divider",
            ("@/components/ui/separator", vec!["Separator"]));
        components.insert("separator",
            ("@/components/ui/separator", vec!["Separator"]));
        components.insert("scroll",
            ("@/components/ui/scroll-area", vec!["ScrollArea"]));
        components.insert("label",
            ("@/components/ui/label", vec!["Label"]));

        // === Feedback: Alert ===
        components.insert("alert",
            ("@/components/ui/alert", vec!["Alert", "AlertTitle", "AlertDescription"]));

        // === Feedback: Toast (Sonner) ===
        components.insert("toast",
            ("@/components/ui/sonner", vec!["Toaster"]));
        components.insert("toaster",
            ("@/components/ui/sonner", vec!["Toaster"]));

        // === Navigation: Dropdown Menu ===
        components.insert("dropdown",
            ("@/components/ui/dropdown-menu", vec!["DropdownMenu", "DropdownMenuTrigger", "DropdownMenuContent", "DropdownMenuItem", "DropdownMenuSeparator", "DropdownMenuLabel"]));
        components.insert("dropdown_menu",
            ("@/components/ui/dropdown-menu", vec!["DropdownMenu", "DropdownMenuTrigger", "DropdownMenuContent"]));
        components.insert("dropdown_trigger",
            ("@/components/ui/dropdown-menu", vec!["DropdownMenuTrigger"]));
        components.insert("dropdown_content",
            ("@/components/ui/dropdown-menu", vec!["DropdownMenuContent"]));
        components.insert("dropdown_item",
            ("@/components/ui/dropdown-menu", vec!["DropdownMenuItem"]));
        components.insert("dropdown_separator",
            ("@/components/ui/dropdown-menu", vec!["DropdownMenuSeparator"]));
        components.insert("dropdown_label",
            ("@/components/ui/dropdown-menu", vec!["DropdownMenuLabel"]));

        // === Overlay: Popover ===
        components.insert("popover",
            ("@/components/ui/popover", vec!["Popover", "PopoverTrigger", "PopoverContent"]));
        components.insert("popover_trigger",
            ("@/components/ui/popover", vec!["PopoverTrigger"]));
        components.insert("popover_content",
            ("@/components/ui/popover", vec!["PopoverContent"]));

        // === Overlay: Sheet (Side Drawer) ===
        components.insert("sheet",
            ("@/components/ui/sheet", vec!["Sheet", "SheetTrigger", "SheetContent", "SheetHeader", "SheetTitle", "SheetDescription", "SheetFooter"]));
        components.insert("sheet_trigger",
            ("@/components/ui/sheet", vec!["SheetTrigger"]));
        components.insert("sheet_content",
            ("@/components/ui/sheet", vec!["SheetContent"]));
        components.insert("sheet_header",
            ("@/components/ui/sheet", vec!["SheetHeader"]));
        components.insert("sheet_title",
            ("@/components/ui/sheet", vec!["SheetTitle"]));
        components.insert("sheet_footer",
            ("@/components/ui/sheet", vec!["SheetFooter"]));

        // === Navigation: Breadcrumb ===
        components.insert("breadcrumb",
            ("@/components/ui/breadcrumb", vec!["Breadcrumb", "BreadcrumbList", "BreadcrumbItem", "BreadcrumbLink", "BreadcrumbSeparator", "BreadcrumbPage"]));
        components.insert("breadcrumb_list",
            ("@/components/ui/breadcrumb", vec!["BreadcrumbList"]));
        components.insert("breadcrumb_item",
            ("@/components/ui/breadcrumb", vec!["BreadcrumbItem"]));
        components.insert("breadcrumb_link",
            ("@/components/ui/breadcrumb", vec!["BreadcrumbLink"]));
        components.insert("breadcrumb_separator",
            ("@/components/ui/breadcrumb", vec!["BreadcrumbSeparator"]));
        components.insert("breadcrumb_page",
            ("@/components/ui/breadcrumb", vec!["BreadcrumbPage"]));

        // === Data Display: Accordion ===
        components.insert("accordion",
            ("@/components/ui/accordion", vec!["Accordion", "AccordionItem", "AccordionTrigger", "AccordionContent"]));
        components.insert("accordion_item",
            ("@/components/ui/accordion", vec!["AccordionItem"]));
        components.insert("accordion_trigger",
            ("@/components/ui/accordion", vec!["AccordionTrigger"]));
        components.insert("accordion_content",
            ("@/components/ui/accordion", vec!["AccordionContent"]));

        // === Overlay: Alert Dialog ===
        components.insert("alert_dialog",
            ("@/components/ui/alert-dialog", vec!["AlertDialog", "AlertDialogTrigger", "AlertDialogContent", "AlertDialogHeader", "AlertDialogFooter", "AlertDialogTitle", "AlertDialogDescription", "AlertDialogAction", "AlertDialogCancel"]));
        components.insert("alert_dialog_trigger",
            ("@/components/ui/alert-dialog", vec!["AlertDialogTrigger"]));
        components.insert("alert_dialog_content",
            ("@/components/ui/alert-dialog", vec!["AlertDialogContent"]));
        components.insert("alert_dialog_header",
            ("@/components/ui/alert-dialog", vec!["AlertDialogHeader"]));
        components.insert("alert_dialog_footer",
            ("@/components/ui/alert-dialog", vec!["AlertDialogFooter"]));
        components.insert("alert_dialog_title",
            ("@/components/ui/alert-dialog", vec!["AlertDialogTitle"]));
        components.insert("alert_dialog_description",
            ("@/components/ui/alert-dialog", vec!["AlertDialogDescription"]));
        components.insert("alert_dialog_action",
            ("@/components/ui/alert-dialog", vec!["AlertDialogAction"]));
        components.insert("alert_dialog_cancel",
            ("@/components/ui/alert-dialog", vec!["AlertDialogCancel"]));
        // Hyphenated versions for AURA tag compatibility
        components.insert("alert-dialog",
            ("@/components/ui/alert-dialog", vec!["AlertDialog"]));
        components.insert("alert-dialog-trigger",
            ("@/components/ui/alert-dialog", vec!["AlertDialogTrigger"]));
        components.insert("alert-dialog-content",
            ("@/components/ui/alert-dialog", vec!["AlertDialogContent"]));
        components.insert("alert-dialog-header",
            ("@/components/ui/alert-dialog", vec!["AlertDialogHeader"]));
        components.insert("alert-dialog-footer",
            ("@/components/ui/alert-dialog", vec!["AlertDialogFooter"]));
        components.insert("alert-dialog-title",
            ("@/components/ui/alert-dialog", vec!["AlertDialogTitle"]));
        components.insert("alert-dialog-description",
            ("@/components/ui/alert-dialog", vec!["AlertDialogDescription"]));
        components.insert("alert-dialog-action",
            ("@/components/ui/alert-dialog", vec!["AlertDialogAction"]));
        components.insert("alert-dialog-cancel",
            ("@/components/ui/alert-dialog", vec!["AlertDialogCancel"]));

        // === Overlay: Command (Command Palette) ===
        components.insert("command",
            ("@/components/ui/command", vec!["Command", "CommandInput", "CommandList", "CommandEmpty", "CommandGroup", "CommandItem", "CommandShortcut", "CommandSeparator"]));
        components.insert("command_input",
            ("@/components/ui/command", vec!["CommandInput"]));
        components.insert("command_list",
            ("@/components/ui/command", vec!["CommandList"]));
        components.insert("command_empty",
            ("@/components/ui/command", vec!["CommandEmpty"]));
        components.insert("command_group",
            ("@/components/ui/command", vec!["CommandGroup"]));
        components.insert("command_item",
            ("@/components/ui/command", vec!["CommandItem"]));
        components.insert("command_shortcut",
            ("@/components/ui/command", vec!["CommandShortcut"]));
        components.insert("command_separator",
            ("@/components/ui/command", vec!["CommandSeparator"]));

        // === Form: Form ===
        components.insert("form",
            ("@/components/ui/form", vec!["Form", "FormField", "FormItem", "FormLabel", "FormControl", "FormDescription", "FormMessage"]));
        components.insert("form_field",
            ("@/components/ui/form", vec!["FormField"]));
        components.insert("form_item",
            ("@/components/ui/form", vec!["FormItem"]));
        components.insert("form_label",
            ("@/components/ui/form", vec!["FormLabel"]));
        components.insert("form_control",
            ("@/components/ui/form", vec!["FormControl"]));
        components.insert("form_description",
            ("@/components/ui/form", vec!["FormDescription"]));
        components.insert("form_message",
            ("@/components/ui/form", vec!["FormMessage"]));

        // === Navigation: Navigation Menu ===
        components.insert("nav_menu",
            ("@/components/ui/navigation-menu", vec!["NavigationMenu", "NavigationMenuList", "NavigationMenuItem", "NavigationMenuLink", "NavigationMenuContent", "NavigationMenuTrigger", "NavigationMenuIndicator"]));
        components.insert("nav_menu_list",
            ("@/components/ui/navigation-menu", vec!["NavigationMenuList"]));
        components.insert("nav_menu_item",
            ("@/components/ui/navigation-menu", vec!["NavigationMenuItem"]));
        components.insert("nav_menu_link",
            ("@/components/ui/navigation-menu", vec!["NavigationMenuLink"]));
        components.insert("nav_menu_content",
            ("@/components/ui/navigation-menu", vec!["NavigationMenuContent"]));
        components.insert("nav_menu_trigger",
            ("@/components/ui/navigation-menu", vec!["NavigationMenuTrigger"]));
        components.insert("nav_menu_indicator",
            ("@/components/ui/navigation-menu", vec!["NavigationMenuIndicator"]));

        // === Navigation: Sidebar ===
        components.insert("sidebar",
            ("@/components/ui/sidebar", vec!["Sidebar", "SidebarHeader", "SidebarContent", "SidebarFooter", "SidebarGroup", "SidebarGroupLabel", "SidebarGroupContent", "SidebarGroupAction", "SidebarMenu", "SidebarMenuItem", "SidebarMenuButton", "SidebarMenuAction", "SidebarMenuBadge", "SidebarMenuSub", "SidebarMenuSubItem", "SidebarMenuSubButton", "SidebarRail", "SidebarSeparator", "SidebarTrigger", "SidebarInset", "SidebarProvider"]));
        components.insert("sidebar_header",
            ("@/components/ui/sidebar", vec!["SidebarHeader"]));
        components.insert("sidebar_content",
            ("@/components/ui/sidebar", vec!["SidebarContent"]));
        components.insert("sidebar_footer",
            ("@/components/ui/sidebar", vec!["SidebarFooter"]));
        components.insert("sidebar_group",
            ("@/components/ui/sidebar", vec!["SidebarGroup"]));
        components.insert("sidebar_group_label",
            ("@/components/ui/sidebar", vec!["SidebarGroupLabel"]));
        components.insert("sidebar_group_content",
            ("@/components/ui/sidebar", vec!["SidebarGroupContent"]));
        components.insert("sidebar_menu",
            ("@/components/ui/sidebar", vec!["SidebarMenu"]));
        components.insert("sidebar_menu_item",
            ("@/components/ui/sidebar", vec!["SidebarMenuItem"]));
        components.insert("sidebar_menu_button",
            ("@/components/ui/sidebar", vec!["SidebarMenuButton"]));
        components.insert("sidebar_trigger",
            ("@/components/ui/sidebar", vec!["SidebarTrigger"]));
        components.insert("sidebar_provider",
            ("@/components/ui/sidebar", vec!["SidebarProvider"]));

        // === Navigation: Stepper ===
        components.insert("stepper",
            ("@/components/ui/stepper", vec!["Stepper", "StepperItem", "StepperTrigger", "StepperIndicator", "StepperTitle", "StepperDescription", "StepperSeparator"]));
        components.insert("stepper_item",
            ("@/components/ui/stepper", vec!["StepperItem"]));
        components.insert("stepper_trigger",
            ("@/components/ui/stepper", vec!["StepperTrigger"]));
        components.insert("stepper_indicator",
            ("@/components/ui/stepper", vec!["StepperIndicator"]));
        components.insert("stepper_title",
            ("@/components/ui/stepper", vec!["StepperTitle"]));
        components.insert("stepper_description",
            ("@/components/ui/stepper", vec!["StepperDescription"]));
        components.insert("stepper_separator",
            ("@/components/ui/stepper", vec!["StepperSeparator"]));

        // ========================================
        // Medium Priority Components
        // ========================================

        // === Calendar ===
        components.insert("calendar",
            ("@/components/ui/calendar", vec!["Calendar", "CalendarCell", "CalendarCellTrigger", "CalendarGrid", "CalendarGridBody", "CalendarGridHead", "CalendarGridRow", "CalendarHeadCell", "CalendarHeader", "CalendarHeading", "CalendarNextButton", "CalendarPrevButton"]));

        // === Carousel ===
        components.insert("carousel",
            ("@/components/ui/carousel", vec!["Carousel", "CarouselContent", "CarouselItem", "CarouselPrevious", "CarouselNext"]));
        components.insert("carousel_content",
            ("@/components/ui/carousel", vec!["CarouselContent"]));
        components.insert("carousel_item",
            ("@/components/ui/carousel", vec!["CarouselItem"]));
        components.insert("carousel_previous",
            ("@/components/ui/carousel", vec!["CarouselPrevious"]));
        components.insert("carousel_prev",
            ("@/components/ui/carousel", vec!["CarouselPrevious"]));
        components.insert("carousel_next",
            ("@/components/ui/carousel", vec!["CarouselNext"]));

        // === Combobox ===
        components.insert("combobox",
            ("@/components/ui/combobox", vec!["Combobox", "ComboboxAnchor", "ComboboxInput", "ComboboxList", "ComboboxEmpty", "ComboboxGroup", "ComboboxItem", "ComboboxSeparator", "ComboboxTrigger"]));
        components.insert("combobox_anchor",
            ("@/components/ui/combobox", vec!["ComboboxAnchor"]));
        components.insert("combobox_input",
            ("@/components/ui/combobox", vec!["ComboboxInput"]));
        components.insert("combobox_list",
            ("@/components/ui/combobox", vec!["ComboboxList"]));
        components.insert("combobox_empty",
            ("@/components/ui/combobox", vec!["ComboboxEmpty"]));
        components.insert("combobox_group",
            ("@/components/ui/combobox", vec!["ComboboxGroup"]));
        components.insert("combobox_item",
            ("@/components/ui/combobox", vec!["ComboboxItem"]));
        components.insert("combobox_trigger",
            ("@/components/ui/combobox", vec!["ComboboxTrigger"]));

        // === Context Menu ===
        components.insert("context_menu",
            ("@/components/ui/context-menu", vec!["ContextMenu", "ContextMenuTrigger", "ContextMenuContent", "ContextMenuGroup", "ContextMenuItem", "ContextMenuCheckboxItem", "ContextMenuRadioGroup", "ContextMenuRadioItem", "ContextMenuSeparator", "ContextMenuLabel", "ContextMenuShortcut", "ContextMenuSub", "ContextMenuSubContent", "ContextMenuSubTrigger"]));
        components.insert("context_menu_trigger",
            ("@/components/ui/context-menu", vec!["ContextMenuTrigger"]));
        components.insert("context_menu_content",
            ("@/components/ui/context-menu", vec!["ContextMenuContent"]));
        components.insert("context_menu_item",
            ("@/components/ui/context-menu", vec!["ContextMenuItem"]));
        components.insert("context_menu_separator",
            ("@/components/ui/context-menu", vec!["ContextMenuSeparator"]));
        components.insert("context_menu_label",
            ("@/components/ui/context-menu", vec!["ContextMenuLabel"]));
        components.insert("context_menu_shortcut",
            ("@/components/ui/context-menu", vec!["ContextMenuShortcut"]));
        components.insert("context_menu_checkbox_item",
            ("@/components/ui/context-menu", vec!["ContextMenuCheckboxItem"]));
        components.insert("context_menu_radio_group",
            ("@/components/ui/context-menu", vec!["ContextMenuRadioGroup"]));
        components.insert("context_menu_radio_item",
            ("@/components/ui/context-menu", vec!["ContextMenuRadioItem"]));
        components.insert("context_menu_sub",
            ("@/components/ui/context-menu", vec!["ContextMenuSub", "ContextMenuSubTrigger", "ContextMenuSubContent"]));
        components.insert("context_menu_sub_trigger",
            ("@/components/ui/context-menu", vec!["ContextMenuSubTrigger"]));
        components.insert("context_menu_sub_content",
            ("@/components/ui/context-menu", vec!["ContextMenuSubContent"]));

        // === Drawer (Vaul) ===
        components.insert("drawer",
            ("@/components/ui/drawer", vec!["Drawer", "DrawerTrigger", "DrawerContent", "DrawerHeader", "DrawerFooter", "DrawerTitle", "DrawerDescription", "DrawerClose"]));
        components.insert("drawer_trigger",
            ("@/components/ui/drawer", vec!["DrawerTrigger"]));
        components.insert("drawer_content",
            ("@/components/ui/drawer", vec!["DrawerContent"]));
        components.insert("drawer_header",
            ("@/components/ui/drawer", vec!["DrawerHeader"]));
        components.insert("drawer_footer",
            ("@/components/ui/drawer", vec!["DrawerFooter"]));
        components.insert("drawer_title",
            ("@/components/ui/drawer", vec!["DrawerTitle"]));
        components.insert("drawer_description",
            ("@/components/ui/drawer", vec!["DrawerDescription"]));
        components.insert("drawer_close",
            ("@/components/ui/drawer", vec!["DrawerClose"]));

        // === Hover Card ===
        components.insert("hover_card",
            ("@/components/ui/hover-card", vec!["HoverCard", "HoverCardTrigger", "HoverCardContent"]));
        components.insert("hover_card_trigger",
            ("@/components/ui/hover-card", vec!["HoverCardTrigger"]));
        components.insert("hover_card_content",
            ("@/components/ui/hover-card", vec!["HoverCardContent"]));

        // === Number Field ===
        components.insert("number_field",
            ("@/components/ui/number-field", vec!["NumberField", "NumberFieldContent", "NumberFieldDecrement", "NumberFieldIncrement", "NumberFieldInput"]));
        components.insert("number_field_input",
            ("@/components/ui/number-field", vec!["NumberFieldInput"]));
        components.insert("number_field_increment",
            ("@/components/ui/number-field", vec!["NumberFieldIncrement"]));
        components.insert("number_field_decrement",
            ("@/components/ui/number-field", vec!["NumberFieldDecrement"]));

        // === Pagination ===
        components.insert("pagination",
            ("@/components/ui/pagination", vec!["Pagination", "PaginationList", "PaginationListItem", "PaginationEllipsis", "PaginationFirst", "PaginationPrev", "PaginationNext", "PaginationLast"]));
        components.insert("pagination_list",
            ("@/components/ui/pagination", vec!["PaginationList"]));
        components.insert("pagination_item",
            ("@/components/ui/pagination", vec!["PaginationListItem"]));
        components.insert("pagination_ellipsis",
            ("@/components/ui/pagination", vec!["PaginationEllipsis"]));
        components.insert("pagination_prev",
            ("@/components/ui/pagination", vec!["PaginationPrev"]));
        components.insert("pagination_next",
            ("@/components/ui/pagination", vec!["PaginationNext"]));
        components.insert("pagination_first",
            ("@/components/ui/pagination", vec!["PaginationFirst"]));
        components.insert("pagination_last",
            ("@/components/ui/pagination", vec!["PaginationLast"]));

        // === Pin Input (OTP) ===
        components.insert("pin_input",
            ("@/components/ui/pin-input", vec!["PinInput", "PinInputGroup", "PinInputSeparator", "PinInputSlot"]));
        components.insert("pin_input_group",
            ("@/components/ui/pin-input", vec!["PinInputGroup"]));
        components.insert("pin_input_slot",
            ("@/components/ui/pin-input", vec!["PinInputSlot"]));
        components.insert("pin_input_separator",
            ("@/components/ui/pin-input", vec!["PinInputSeparator"]));

        // === Tags Input ===
        components.insert("tags_input",
            ("@/components/ui/tags-input", vec!["TagsInput", "TagsInputInput", "TagsInputItem", "TagsInputItemDelete", "TagsInputItemText"]));
        components.insert("tags_input_field",
            ("@/components/ui/tags-input", vec!["TagsInputInput"]));
        components.insert("tags_input_item",
            ("@/components/ui/tags-input", vec!["TagsInputItem"]));
        components.insert("tags_input_delete",
            ("@/components/ui/tags-input", vec!["TagsInputItemDelete"]));

        // === Toggle Group ===
        components.insert("toggle_group",
            ("@/components/ui/toggle-group", vec!["ToggleGroup", "ToggleGroupItem"]));
        components.insert("toggle_group_item",
            ("@/components/ui/toggle-group", vec!["ToggleGroupItem"]));

        // ========================================
        // Low Priority Components
        // ========================================

        // === Aspect Ratio ===
        components.insert("aspect_ratio",
            ("@/components/ui/aspect-ratio", vec!["AspectRatio"]));

        // === Button Group ===
        components.insert("button_group",
            ("@/components/ui/button-group", vec!["ButtonGroup"]));

        // === Chart ===
        components.insert("chart",
            ("@/components/ui/chart", vec!["ChartContainer", "ChartTooltip", "ChartLegend", "ChartStyle"]));

        // === Collapsible ===
        components.insert("collapsible",
            ("@/components/ui/collapsible", vec!["Collapsible", "CollapsibleTrigger", "CollapsibleContent"]));
        components.insert("collapsible_trigger",
            ("@/components/ui/collapsible", vec!["CollapsibleTrigger"]));
        components.insert("collapsible_content",
            ("@/components/ui/collapsible", vec!["CollapsibleContent"]));

        // === Input Group ===
        components.insert("input_group",
            ("@/components/ui/input-group", vec!["InputGroup", "InputGroupText"]));

        // === Input OTP ===
        components.insert("input_otp",
            ("@/components/ui/input-otp", vec!["InputOTP", "InputGroup", "InputOTPSlot", "InputOTPSeparator"]));

        // === Kbd (Keyboard) ===
        components.insert("kbd",
            ("@/components/ui/kbd", vec!["Kbd"]));

        // === Menubar ===
        components.insert("menubar",
            ("@/components/ui/menubar", vec!["Menubar", "MenubarMenu", "MenubarTrigger", "MenubarContent", "MenubarItem", "MenubarSeparator", "MenubarLabel", "MenubarCheckboxItem", "MenubarRadioGroup", "MenubarRadioItem", "MenubarShortcut", "MenubarSub", "MenubarSubTrigger", "MenubarSubContent"]));
        components.insert("menubar_menu",
            ("@/components/ui/menubar", vec!["MenubarMenu"]));
        components.insert("menubar_trigger",
            ("@/components/ui/menubar", vec!["MenubarTrigger"]));
        components.insert("menubar_content",
            ("@/components/ui/menubar", vec!["MenubarContent"]));
        components.insert("menubar_item",
            ("@/components/ui/menubar", vec!["MenubarItem"]));
        components.insert("menubar_separator",
            ("@/components/ui/menubar", vec!["MenubarSeparator"]));
        components.insert("menubar_label",
            ("@/components/ui/menubar", vec!["MenubarLabel"]));

        // === Native Select ===
        components.insert("native_select",
            ("@/components/ui/native-select", vec!["NativeSelect", "NativeSelectOption", "NativeSelectGroup", "NativeSelectLabel"]));

        // === Range Calendar ===
        components.insert("range_calendar",
            ("@/components/ui/range-calendar", vec!["RangeCalendar", "RangeCalendarCell", "RangeCalendarCellTrigger", "RangeCalendarGrid", "RangeCalendarGridBody", "RangeCalendarGridHead", "RangeCalendarGridRow", "RangeCalendarHeadCell", "RangeCalendarHeader", "RangeCalendarHeading", "RangeCalendarNextButton", "RangeCalendarPrevButton"]));

        // === Resizable ===
        components.insert("resizable",
            ("@/components/ui/resizable", vec!["ResizablePanelGroup", "ResizablePanel", "ResizableHandle"]));
        components.insert("resizable_panel",
            ("@/components/ui/resizable", vec!["ResizablePanel"]));
        components.insert("resizable_handle",
            ("@/components/ui/resizable", vec!["ResizableHandle"]));

        // === Auto Complete ===
        components.insert("autocomplete",
            ("@/components/ui/auto-complete", vec!["AutoComplete", "AutoCompleteContent", "AutoCompleteEmpty", "AutoCompleteGroup", "AutoCompleteGroupHeading", "AutoCompleteItem", "AutoCompleteInput", "AutoCompleteList", "AutoCompleteTrigger"]));
        components.insert("autocomplete_input",
            ("@/components/ui/auto-complete", vec!["AutoCompleteInput"]));
        components.insert("autocomplete_item",
            ("@/components/ui/auto-complete", vec!["AutoCompleteItem"]));
        components.insert("autocomplete_list",
            ("@/components/ui/auto-complete", vec!["AutoCompleteList"]));
        components.insert("autocomplete_empty",
            ("@/components/ui/auto-complete", vec!["AutoCompleteEmpty"]));

        Self { components }
    }

    /// Normalize tag name for lookup (convert kebab-case to snake_case)
    fn normalize_tag(tag: &str) -> String {
        tag.replace('-', "_")
    }

    /// Get shadcn-vue component info for a tag
    pub fn get(&self, tag: &str) -> Option<(&'static str, &Vec<&'static str>)> {
        let normalized = Self::normalize_tag(tag);
        self.components.get(normalized.as_str()).map(|(path, names)| (*path, names))
    }

    /// Check if tag has a shadcn-vue component
    pub fn has_component(&self, tag: &str) -> bool {
        let normalized = Self::normalize_tag(tag);
        self.components.contains_key(normalized.as_str())
    }

    /// Get the primary component name for a tag (first in the list)
    pub fn primary_component(&self, tag: &str) -> Option<&'static str> {
        let normalized = Self::normalize_tag(tag);
        self.components.get(normalized.as_str()).and_then(|(_, names)| names.first().copied())
    }
}

#[allow(deprecated)]
impl Default for ShadcnRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Vue Generator
// ============================================================================

/// Generation mode for Vue output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VueMode {
    /// Plain HTML with Tailwind CSS classes
    #[default]
    Plain,
    /// shadcn-vue components with accessibility built-in
    Shadcn,
    /// Self-contained library widgets (Plan 331): each primitive emits an
    /// independent SFC importing `reka-ui` directly, never `@/components/ui/*`.
    Library,
}

/// Vue3 SFC generator
pub struct VueGenerator {
    /// Current widget name
    current_widget: Option<String>,

    /// Collected imports
    imports: Vec<String>,

    /// State variable names (for ref() detection)
    state_names: Vec<String>,

    /// Prop names (for defineProps — no .value suffix needed)
    prop_names: Vec<String>,

    /// Prop name → TS type (for computed expression type inference)
    prop_types: HashMap<String, String>,

    /// State variable name → TS type (for computed expression type inference)
    state_types: HashMap<String, String>,

    /// Store dependencies from `use store:` (Plan 351)
    store_deps: Vec<String>,

    /// Whether the project depends on @autodown/editor (from pac.at npm_deps).
    /// Drives R003 validation + main.ts CSS import.
    uses_autodown: bool,

    /// Warnings from the last validate_sfc run (Plan 361).
    pub last_validation_warnings: Vec<crate::ui_gen::validators::ValidationWarning>,

    /// Event handler definitions (name, body, is_async)
    handlers: Vec<(String, String, bool)>,

    /// Event names for emit
    emit_events: Vec<String>,

    /// Whether emit is needed
    has_emit: bool,

    /// Component references (other widgets)
    component_refs: Vec<String>,

    /// Lucide icon components used (for import collection)
    lucide_icons: HashSet<String>,

    /// Tailwind classes for wrapper
    wrapper_classes: String,

    /// Generation mode (Plain or Shadcn)
    mode: VueMode,

    /// Unified widget registry (replaces ShadcnRegistry)
    #[allow(dead_code)]
    widget_registry: WidgetRegistry,

    /// Track which shadcn-vue components are used (for import collection)
    shadcn_components_used: HashSet<String>,

    /// Whether to output TypeScript (Plan 100: a2js → a2ts)
    use_typescript: bool,

    /// Counter for unique previewcard IDs
    previewcard_counter: usize,

    /// Data for each previewcard (id, auto_code, vue_code)
    previewcard_data: Vec<PreviewCardData>,

    /// Whether copyCode function is needed
    needs_copy_code: bool,

    /// Counter for unique codeblock IDs
    codeblock_counter: usize,

    /// Data for each codeblock (id, code, lang)
    codeblock_data: Vec<CodeBlockData>,

    /// Whether router is needed (has outlet, link, or nav() calls) - Plan 105
    needs_router: bool,
    /// Whether useRoute is needed (has route.param/query/path access) - Plan 235
    needs_route: bool,

    /// API functions used in handlers (Plan 132)
    api_functions_used: HashSet<String>,

    /// Project-specific API function names loaded from dist/.api_functions
    project_api_functions: Vec<String>,

    /// Handler names actually referenced in the template
    used_handlers: HashSet<String>,

    /// Whether the widget has an isDark state var (dark mode toggle)
    has_dark_mode: bool,
    /// Name of the dark mode state variable (e.g. "isDark" or "dark_mode")
    dark_mode_var: Option<String>,

    /// Whether theme-toggle component is used
    use_theme_toggle: bool,

    /// Whether CurveType from @unovis/ts is needed (for chart curve-type props)
    use_curve_type: bool,

    /// Names of known sub-widgets in the same project (e.g. "Sidebar", "EditorPanel")
    /// When a tag matches one of these, skip shadcn component mapping and treat as custom component
    known_sub_widgets: HashSet<String>,

    /// Current for-loop variable name (e.g., "note") — used to pass loop var as event arg
    /// When inside a `for note in .notes { ... }`, this is set to Some("note")
    current_loop_var: Option<String>,

    /// True when `current_loop_var` is the loop INDEX variable
    /// (`for i, note in .notes` → current_loop_var = "i", an int).
    /// Index vars are primitive ints, so the auto-:key must not emit `i?.id`.
    current_loop_var_is_index: bool,

    /// Per-widget key counter — increments for every component usage in the template.
    /// Produces stable unique keys like 'AutoDownEditor-1', 'NavTree-2', etc. so that
    /// two components with the same name (e.g. in different v-if branches) don't collide.
    widget_key_counter: usize,

    /// Handlers that need a loop-id parameter (e.g., "SelectNote" needs `i: any`)
    /// Populated during template generation, consumed during script generation.
    /// Maps handler name → loop variable name (e.g., "SelectNote" → "i").
    loop_param_handlers: HashMap<String, String>,

    /// Whether to generate handleChildDelete function (auto-wired when sub-widget emits Delete)
    needs_child_delete_handler: bool,

    /// Whether API functions were explicitly imported via `use back.api: ...`
    /// When true, skip AST scanning and use the explicit import list
    explicit_api_imports: bool,

    /// document/window-level event listeners declared in the view via
    /// `.window` / `.document` event modifiers (e.g. `onmousemove.window`).
    /// Populated during template generation, consumed during script generation
    /// to emit addEventListener/removeEventListener pairs.
    global_listeners: Vec<GlobalListener>,

    /// Template refs declared in the view via `ref: "menuEl"`.
    /// Populated during template generation, consumed during script generation
    /// to emit `const menuEl = ref<HTMLElement | null>(null)` declarations.
    template_refs: Vec<String>,

    /// Subset of `template_refs` attached to child components (not DOM
    /// elements) via `ref: "canvasRef"` on a sub-widget instantiation.
    /// Declared as `ref<any>(null)` since the child's defineExpose surface
    /// is unknown to the parent.
    component_ref_names: HashSet<String>,

    /// External Vue components declared via the widget-level
    /// `use { component: Name from "..." }` block. Keyed by every accepted
    /// view tag form (exact name, snake_case, kebab-case) so `FancyBadge`,
    /// `fancy_badge`, and `fancy-badge` all resolve to the component.
    ext_components: HashMap<String, ExtComponent>,

    /// Import lines from the widget `use { ... }` block, as
    /// (imported symbol names, is_component, line). `is_component` entries
    /// are dropped at emission time when the built-in registry already
    /// imported the same symbol (avoids duplicate bindings).
    ext_import_lines: Vec<(Vec<String>, bool, String)>,

    /// Composables from `use { composable: ... }` to call once at
    /// `<script setup>` top level: (local const name, callee name).
    ext_composables: Vec<(String, String)>,
}

/// A Vue component declared in a widget-level `use { component: ... }` block.
#[derive(Debug, Clone)]
struct ExtComponent {
    /// Component name as imported (e.g. "FancyBadge").
    name: String,
}

/// A document/window-level event listener (Plan: generic DOM events).
///
/// Declared in the view with a `.window` or `.document` event modifier:
/// `onmousemove.window: .DragMove($event)`. Instead of a template attribute,
/// the generator emits `target.addEventListener(...)` in onMounted and the
/// matching `removeEventListener(...)` in onUnmounted.
#[derive(Debug, Clone)]
struct GlobalListener {
    /// "window" or "document"
    target: String,
    /// DOM event name (e.g. "mousemove", "wheel")
    event: String,
    /// Listener expression: either the handler function reference or the
    /// name of a generated wrapper function (when prevent/stop/args apply)
    listener: String,
    /// capture phase flag (mirrored on removeEventListener)
    capture: bool,
    /// Explicit passive option (`passive: false` is required for
    /// preventDefault on document-level wheel/touch listeners in Chrome)
    passive: Option<bool>,
    /// Wrapper function source, emitted once before the onMounted block
    wrapper: Option<String>,
}

/// Data for generating interactive preview cards
#[derive(Debug, Clone)]
struct PreviewCardData {
    /// Unique identifier (e.g., "preview", "variants")
    id: String,
    /// Auto (AURA) source code
    auto_code: String,
    /// Vue.js source code
    vue_code: String,
}

/// Data for generating code blocks with copy button
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CodeBlockData {
    /// Unique identifier (e.g., "install-button", "install-card")
    id: String,
    /// Code content
    code: String,
    /// Language (e.g., "bash", "typescript")
    lang: String,
}

impl VueGenerator {
    /// Create a new Vue generator (Plain Tailwind mode, TypeScript output)
    pub fn new() -> Self {
        Self {
            current_widget: None,
            imports: Vec::new(),
            state_names: Vec::new(),
            prop_names: Vec::new(),
            prop_types: HashMap::new(),
            state_types: HashMap::new(),
            store_deps: Vec::new(),
            uses_autodown: false,
            last_validation_warnings: Vec::new(),
            handlers: Vec::new(),
            emit_events: Vec::new(),
            has_emit: false,
            component_refs: Vec::new(),
            lucide_icons: HashSet::new(),
            wrapper_classes: String::new(),
            mode: VueMode::Plain,
            widget_registry: WidgetRegistry::with_defaults(),
            shadcn_components_used: HashSet::new(),
            use_typescript: true,  // Plan 100: TypeScript by default
            previewcard_counter: 0,
            previewcard_data: Vec::new(),
            needs_copy_code: false,
            codeblock_counter: 0,
            codeblock_data: Vec::new(),
            needs_router: false,
            needs_route: false,
            api_functions_used: HashSet::new(),
            project_api_functions: {
                // DEPRECATED: Env var fallback for backward compatibility.
                // New code should use `with_project_api_functions()` from explicit imports.
                let val = std::env::var("AUTO_API_FUNCTIONS").unwrap_or_default();
                val.split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.trim().to_string())
                    .collect()
            },
            used_handlers: HashSet::new(),
            has_dark_mode: false,
            dark_mode_var: None,
            use_theme_toggle: false,
            use_curve_type: false,
            known_sub_widgets: HashSet::new(),
            current_loop_var: None,
            current_loop_var_is_index: false,
            widget_key_counter: 0,
            loop_param_handlers: HashMap::new(),
            needs_child_delete_handler: false,
            explicit_api_imports: false,
            global_listeners: Vec::new(),
            template_refs: Vec::new(),
            component_ref_names: HashSet::new(),
            ext_components: HashMap::new(),
            ext_import_lines: Vec::new(),
            ext_composables: Vec::new(),
        }
    }

    /// Set known sub-widget names (to avoid shadcn name collisions)
    pub fn with_sub_widgets(mut self, names: Vec<String>) -> Self {
        self.known_sub_widgets = names.into_iter().collect();
        self
    }

    /// Set project-specific API function names (from explicit `use back.api: ...` imports)
    /// When set via this method (from explicit imports), skip AST scanning and use this list directly.
    pub fn with_project_api_functions(mut self, functions: Vec<String>) -> Self {
        if !functions.is_empty() {
            self.explicit_api_imports = true;
        }
        self.project_api_functions = functions;
        self
    }

    /// Set store dependencies from `use store:` declarations (Plan 351).
    pub fn with_store_deps(mut self, deps: Vec<String>) -> Self {
        self.store_deps = deps;
        self
    }

    /// Mark whether the project depends on @autodown/editor (Plan 361).
    /// Drives R003 validation (CSS import check) and is consumed by
    /// auto-man's generate_main_ts to inject the stylesheet import.
    pub fn with_uses_autodown(mut self, uses: bool) -> Self {
        self.uses_autodown = uses;
        self
    }

    /// Check if a name is a known API function (static list OR project-specific)
    fn is_api_function(&self, name: &str) -> bool {
        Self::API_FUNCTIONS.contains(&name) || self.project_api_functions.iter().any(|f| f == name)
    }

    /// Get the combined list of all known API function names
    fn all_api_functions(&self) -> Vec<String> {
        let mut fns: Vec<String> = Self::API_FUNCTIONS.iter().map(|s| s.to_string()).collect();
        fns.extend(self.project_api_functions.iter().cloned());
        fns
    }

    /// Create a new Vue generator in shadcn-vue mode
    pub fn new_shadcn() -> Self {
        Self {
            mode: VueMode::Shadcn,
            widget_registry: WidgetRegistry::with_defaults(),
            ..Self::new()
        }
    }

    /// Create a new Vue generator in library mode (Plan 331): emits
    /// self-contained per-widget SFCs backed by `reka-ui`.
    pub fn new_library() -> Self {
        Self {
            mode: VueMode::Library,
            widget_registry: WidgetRegistry::with_defaults(),
            ..Self::new()
        }
    }

    /// Set the generation mode
    pub fn with_mode(mut self, mode: VueMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set whether to use TypeScript output (Plan 100)
    pub fn with_typescript(mut self, use_typescript: bool) -> Self {
        self.use_typescript = use_typescript;
        self
    }

    /// Check if using shadcn-vue mode
    pub fn is_shadcn(&self) -> bool {
        self.mode == VueMode::Shadcn
    }

    /// Check if using library mode (Plan 331)
    pub fn is_library(&self) -> bool {
        self.mode == VueMode::Library
    }

    /// Check if outputting TypeScript (Plan 100)
    pub fn is_typescript(&self) -> bool {
        self.use_typescript
    }

    /// Generate a self-contained SFC for a single primitive widget (Plan 331).
    ///
    /// Emits a standalone `.vue` file backed by `reka-ui` (never
    /// `@/components/ui/*`), driven by the widget's library template.
    pub fn generate_widget_sfc(&mut self, name: &str) -> GenResult<String> {
        let tpl = library_template(name)
            .ok_or_else(|| GenError::UnknownWidget(name.to_string()))?;
        Ok(format!(
            "{header}\n<script setup lang=\"ts\">\n{script}\n</script>\n\n<template>\n{template}\n</template>\n",
            header = attribution_header(name),
            script = tpl.script,
            template = tpl.template,
        ))
    }

    /// Emit the per-widget support files (relative path, contents) that the
    /// generated SFC depends on, so a copied component is self-contained.
    pub fn generate_widget_support_files(&self, name: &str) -> Vec<(String, String)> {
        let pascal = pascal_case(name);
        // Collect every `.vue` file this widget emits: the primary SFC plus any
        // companion `.vue` files declared in `extra_support_files` (composite
        // widgets like card/dialog/tabs ship several SFCs in one directory).
        let mut vue_files = vec![format!("{pascal}.vue")];
        let mut extras: Vec<(String, String)> = Vec::new();
        if let Some(tpl) = library_template(name) {
            for (n, c) in tpl.extra_support_files.iter() {
                let n = n.to_string();
                if n.ends_with(".vue") {
                    vue_files.push(n.clone());
                }
                extras.push((n, c.to_string()));
            }
        }
        // index.ts re-exports every emitted `.vue` by its PascalCase basename.
        let mut index = String::new();
        for file in &vue_files {
            let stem = file.trim_end_matches(".vue");
            index.push_str(&format!(
                "export {{ default as {stem} }} from './{file}'\n"
            ));
        }
        let mut files = vec![("index.ts".to_string(), index)];
        files.extend(extras);
        files
    }

    /// Files shared by every library widget, written once at the registry root
    /// (Plan 331). Currently the `cn` class-merge helper that all generated
    /// SFCs import as `../utils`. `auto ui build` emits these alongside the
    /// widget directories; `auto-ui add` copies them into the consumer root.
    pub fn library_shared_files(&self) -> Vec<(&'static str, &'static str)> {
        vec![("utils.ts", LIBRARY_UTILS_TS)]
    }

    /// Reset state for new widget
    fn reset(&mut self) {
        self.imports.clear();
        self.state_names.clear();
        self.prop_names.clear();
        self.prop_types.clear();
        self.state_types.clear();
        self.handlers.clear();
        self.emit_events.clear();
        self.has_emit = false;
        self.component_refs.clear();
        self.lucide_icons.clear();
        self.wrapper_classes.clear();
        self.current_loop_var = None;
        self.current_loop_var_is_index = false;
        self.widget_key_counter = 0;
        self.loop_param_handlers.clear();
        self.needs_child_delete_handler = false;
        // NOTE: explicit_api_imports is NOT reset — it's a config-level setting from with_project_api_functions()
        self.shadcn_components_used.clear();
        self.previewcard_counter = 0;
        self.previewcard_data.clear();
        self.needs_copy_code = false;
        self.codeblock_counter = 0;
        self.codeblock_data.clear();
        self.needs_router = false;
        self.needs_route = false;
        self.api_functions_used.clear();
        // NOTE: project_api_functions is NOT cleared on reset — it's config-level,
        // loaded once from AUTO_API_FUNCTIONS env var, and persists across widget generation.
        self.used_handlers.clear();
        self.has_dark_mode = false;
        self.dark_mode_var = None;
        self.use_theme_toggle = false;
        self.global_listeners.clear();
        self.template_refs.clear();
        self.ext_components.clear();
        self.ext_import_lines.clear();
        self.ext_composables.clear();
    }

    /// Convert kebab-case icon name to PascalCase Lucide component name
    fn kebab_to_pascal(s: &str) -> String {
        s.split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect()
    }

    // ====================================================================
    // Widget `use { ... }` external TS/Vue imports (escape hatch)
    // ====================================================================

    /// True when a widget `use { ... }` import path refers to a
    /// project-local file (copied into the generated project under
    /// `src/ext/` by auto-man) rather than an npm package specifier.
    fn ext_is_local_path(path: &str) -> bool {
        path.starts_with('.')
            || path.starts_with('/')
            || path.ends_with(".vue")
            || path.ends_with(".ts")
            || path.ends_with(".tsx")
            || path.ends_with(".js")
            || path.ends_with(".mjs")
    }

    /// Map a declared import path to the specifier used in the generated
    /// SFC. npm specifiers pass through unchanged; local files are copied
    /// by auto-man into `src/ext/<path>` (preserving the
    /// project-root-relative layout so sibling relative imports keep
    /// working) and imported through the `@` alias. TypeScript/JavaScript
    /// extensions are dropped (bundler resolution); `.vue` is kept.
    fn ext_import_specifier(path: &str) -> String {
        if !Self::ext_is_local_path(path) {
            return path.to_string();
        }
        let rel = path.trim_start_matches("./").trim_start_matches('/');
        let stem = rel
            .strip_suffix(".tsx")
            .or_else(|| rel.strip_suffix(".ts"))
            .or_else(|| rel.strip_suffix(".mjs"))
            .or_else(|| rel.strip_suffix(".js"))
            .unwrap_or(rel);
        format!("@/ext/{}", stem)
    }

    /// The view tag spellings accepted for an external component: the
    /// declared name itself plus its snake_case and kebab-case forms
    /// (`FancyBadge` → `FancyBadge`, `fancy_badge`, `fancy-badge`).
    fn ext_tag_keys(name: &str) -> Vec<String> {
        let mut snake = String::new();
        for (i, c) in name.chars().enumerate() {
            if c.is_ascii_uppercase() && i > 0 {
                snake.push('_');
            }
            snake.push(c.to_ascii_lowercase());
        }
        let kebab = snake.replace('_', "-");
        let mut keys = vec![name.to_string(), snake.clone()];
        if kebab != snake {
            keys.push(kebab);
        }
        keys
    }

    /// `useMenuBounds` → `menuBounds` (strip the `use` prefix, lowercase
    /// the first letter) — the local const a composable's return value is
    /// bound to at `<script setup>` top level.
    fn ext_composable_local_name(callee: &str) -> String {
        let base = callee.strip_prefix("use").unwrap_or(callee);
        let mut chars = base.chars();
        match chars.next() {
            Some(first) => format!("{}{}", first.to_ascii_lowercase(), chars.as_str()),
            None => base.to_string(),
        }
    }

    /// Register a widget's `use { ... }` external imports: build the
    /// tag → component lookup used while generating the view tree, the
    /// import lines emitted into `<script setup>`, and the composable
    /// call list. Called from `generate_sfc` before template generation.
    fn register_ext_imports(&mut self, widget: &AuraWidget) {
        for imp in &widget.ext_imports {
            let specifier = Self::ext_import_specifier(&imp.path);
            let symbols: Vec<String> = imp.symbols.iter().map(|s| s.as_str().to_string()).collect();
            match imp.kind {
                crate::ast::ExtImportKind::Fn | crate::ast::ExtImportKind::Composable => {
                    self.ext_import_lines.push((
                        symbols.clone(),
                        false,
                        format!("import {{ {} }} from '{}'\n", symbols.join(", "), specifier),
                    ));
                    if imp.kind == crate::ast::ExtImportKind::Composable {
                        for sym in &symbols {
                            self.ext_composables
                                .push((Self::ext_composable_local_name(sym), sym.clone()));
                        }
                    }
                }
                crate::ast::ExtImportKind::Component => {
                    for sym in &symbols {
                        // Local `.vue` files are default-exported; everything
                        // else (npm packages, .ts modules) uses named exports.
                        let line = if imp.path.ends_with(".vue") {
                            format!("import {} from '{}'\n", sym, specifier)
                        } else {
                            format!("import {{ {} }} from '{}'\n", sym, specifier)
                        };
                        self.ext_import_lines.push((vec![sym.clone()], true, line));
                        let comp = ExtComponent { name: sym.clone() };
                        for key in Self::ext_tag_keys(sym) {
                            self.ext_components.insert(key, comp.clone());
                        }
                    }
                }
            }
        }
    }

    /// Get Tailwind color classes for category section
    fn category_color_classes(color: &str) -> (&'static str, &'static str) {
        match color {
            "blue" => ("bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-200 dark:border-blue-800", "bg-blue-500"),
            "emerald" => ("bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-200 dark:border-emerald-800", "bg-emerald-500"),
            "amber" => ("bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-200 dark:border-amber-800", "bg-amber-500"),
            "purple" => ("bg-purple-500/10 text-purple-600 dark:text-purple-400 border-purple-200 dark:border-purple-800", "bg-purple-500"),
            "rose" => ("bg-rose-500/10 text-rose-600 dark:text-rose-400 border-rose-200 dark:border-rose-800", "bg-rose-500"),
            _ => ("bg-gray-500/10 text-gray-600 dark:text-gray-400 border-gray-200 dark:border-gray-800", "bg-gray-500"),
        }
    }

    /// Generate category-section HTML (component grid with heading)
    fn generate_category_section_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "  ".repeat(indent);
        let name = props.get("name").and_then(|v| self.extract_string_value(v)).unwrap_or("Category");
        let color = props.get("color").and_then(|v| self.extract_string_value(v)).unwrap_or("gray");
        let count = props.get("count")
            .and_then(|v| self.extract_int_value(v).map(|n| n.to_string()))
            .or_else(|| props.get("count").and_then(|v| self.extract_string_value(v)).map(|s| s.to_string()))
            .unwrap_or_default();

        let (item_classes, dot_class) = Self::category_color_classes(color);
        self.lucide_icons.insert("ArrowRight".to_string());

        let mut html = String::new();
        html.push_str(&format!("{}<div>\n", ind));
        html.push_str(&format!("{}  <div class=\"flex items-center gap-2 mb-4\">\n", ind));
        html.push_str(&format!("{}    <span class=\"h-2.5 w-2.5 rounded-full {}\" />\n", ind, dot_class));
        html.push_str(&format!("{}    <h2 class=\"text-sm font-semibold uppercase tracking-wider text-muted-foreground\">{}</h2>\n", ind, name));
        html.push_str(&format!("{}    <span class=\"text-xs text-muted-foreground/60\">({})</span>\n", ind, count));
        html.push_str(&format!("{}  </div>\n", ind));
        html.push_str(&format!("{}  <div class=\"grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3\">\n", ind));

        let has_search = self.state_names.contains(&"searchQuery".to_string());

        for child in children {
            if let AuraNode::Element { tag: child_tag, props: child_props, .. } = child {
                if child_tag == "component-card" || child_tag == "component_card" || child_tag == "componentcard" {
                    let to = child_props.get("to").and_then(|v| self.extract_string_value(v)).unwrap_or("#");
                    let card_name = child_props.get("name").and_then(|v| self.extract_string_value(v)).unwrap_or("");
                    let desc = child_props.get("desc").and_then(|v| self.extract_string_value(v)).unwrap_or("");
                    let icon_name = child_props.get("icon").and_then(|v| self.extract_string_value(v)).unwrap_or("");
                    let lucide_component = Self::kebab_to_pascal(icon_name);
                    self.lucide_icons.insert(lucide_component.clone());

                    let vshow = if has_search {
                        format!(r#" v-show="!searchQuery || '{}'.toLowerCase().includes(searchQuery.toLowerCase()) || '{}'.toLowerCase().includes(searchQuery.toLowerCase())""#, card_name, desc)
                    } else {
                        String::new()
                    };

                    html.push_str(&format!(
                        r#"{}    <router-link to="{}"{} class="group flex items-start gap-3 rounded-xl border p-4 text-left transition-all duration-200 hover:shadow-md hover:-translate-y-0.5 hover:border-primary/30 bg-card">
{}      <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border {}">
{}        <{} class="h-5 w-5" />
{}      </div>
{}      <div class="min-w-0">
{}        <div class="font-medium text-sm truncate">{}</div>
{}        <div class="text-xs text-muted-foreground truncate">{}</div>
{}      </div>
{}      <ArrowRight class="h-4 w-4 ml-auto shrink-0 text-muted-foreground opacity-0 -translate-x-2 transition-all group-hover:opacity-100 group-hover:translate-x-0" />
{}    </router-link>
"#,
                        ind, to, vshow,
                        ind, item_classes,
                        ind, lucide_component,
                        ind,
                        ind,
                        ind, card_name,
                        ind, desc,
                        ind,
                        ind,
                        ind
                    ));
                }
            }
        }

        html.push_str(&format!("{}  </div>\n", ind));
        html.push_str(&format!("{}</div>\n", ind));
        Ok(html)
    }

    /// Generate complete Vue3 SFC
    pub fn generate_sfc(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.current_widget = Some(widget.name.clone());
        self.reset();
        // Widget `use { ... }` external imports — must be registered before
        // template generation so view tags resolve to external components.
        self.register_ext_imports(widget);

        // Detect dark mode: check widget state vars, view tree, or handler names
        self.has_dark_mode = widget.state_vars.iter().any(|s| s.name == "isDark" || s.name == "dark_mode");
        // Determine the dark mode state variable name for template binding
        self.dark_mode_var = widget.state_vars.iter()
            .find(|s| s.name == "isDark" || s.name == "dark_mode")
            .map(|s| s.name.clone());
        // Also check if ToggleDarkMode handler exists (D1 fix: removed
        // format!("{:?}", widget.view_tree) which caused OOM on large
        // view trees with Expr::If nodes — handler check is sufficient)
        if !self.has_dark_mode {
            // Handler keys are stored as ".ToggleDarkMode" (with leading dot)
            let has_toggle = widget.handlers.contains_key(".ToggleDarkMode")
                || widget.lifecycle.iter().any(|l| l.name.contains("DarkMode") || l.name.contains("dark_mode"));
            if has_toggle {
                self.has_dark_mode = true;
                self.dark_mode_var = Some("dark_mode".to_string());
            }
        }

        // Pre-populate state_names so expr_to_js recognizes refs during template generation
        for state in &widget.state_vars {
            self.state_names.push(state.name.clone());
            self.state_types.insert(state.name.clone(), Self::auto_type_to_ts_type(&state.type_info));
        }

        // Register prop names (props are NOT refs — no .value suffix in script)
        for prop in &widget.props {
            self.prop_names.push(prop.name.clone());
            self.prop_types.insert(prop.name.clone(), Self::auto_type_to_ts_type(&prop.type_info));
        }

        // Plan 351: 'store' is a local const (from `const store = reactive(useXxxStore())`).
        // NOT a prop — must not be in prop_names, or ts_adapter rewrites .store.xxx
        // to props.store.xxx (wrong). As a bare ident, it passes through correctly
        // (not in state_names → no .value suffix; not in prop_names → no props. prefix).

        // Activate emit generation for sub-widgets that have messages
        if !widget.messages.is_empty() {
            self.has_emit = true;
            for msg in &widget.messages {
                for variant in &msg.variants {
                    self.emit_events.push(variant.name.clone());
                }
            }
        }

        // Generate template first to collect shadcn components used and detect Outlet/Link
        let template = self.generate_template(&widget.view_tree)?;

        // Plan 105: Check handlers for NavCall
        if self.widget_needs_router(widget) {
            self.needs_router = true;
        }
        // Plan 235: Check handlers for route access
        if Self::widget_needs_route(widget) {
            self.needs_route = true;
        }
        // Plan 235: Pre-analyze handlers for route access and navigation
        // (ts_adapter builtins like router.param() emit useRoute() which we need to import)
        for payload in widget.handlers.values() {
            if let Ok(body) = self.generate_handler_body(payload) {
                if body.contains("useRoute") {
                    self.needs_route = true;
                }
            }
            // Also check raw AST for router.push / router.replace
            match payload {
                LogicPayload::AstStmts(stmts) => {
                    if crate::ui_gen::ts_adapter::stmts_have_router_nav(stmts) {
                        self.needs_router = true;
                    }
                }
                _ => {}
            }
        }

        // Then generate script (which can now include shadcn imports and router)
        let script = self.generate_script(widget)?;
        let style = self.generate_style();

        // Widget-level native CSS (`style { ... }` block) — captured verbatim
        // by the lexer and emitted unchanged into a dedicated `<style scoped>`
        // block. Never interpreted, never merged with the generated <style>.
        let scoped_style = match &widget.style_css {
            Some(css) => format!("\n<style scoped>\n{}</style>\n", css),
            None => String::new(),
        };

        // Plan 100: Add lang="ts" for TypeScript output
        let script_tag = if self.use_typescript {
            r#"<script setup lang="ts">"#
        } else {
            r#"<script setup>"#
        };

        let sfc = format!(
            r#"<!-- {} component - Auto-generated from Auto language -->
{}
{}
</script>

<template>
{}
</template>

<style>
{}
</style>
{}"#,
            widget.name, script_tag, script, template, style, scoped_style
        );

        // Plan 361: Post-generation validation. Run all rules against the SFC
        // and stash warnings so callers (generate_component_from_file, auto-man)
        // can print/log them. Non-fatal — generation always succeeds.
        let ctx = crate::ui_gen::validators::ValidationContext {
            store_deps: self.store_deps.clone(),
            uses_autodown: self.uses_autodown,
            used_handlers: self.used_handlers.iter().cloned().collect(),
            strict: false,
        };
        let warnings = crate::ui_gen::validate_sfc(&sfc, &widget.name, &ctx);
        self.last_validation_warnings = warnings;

        Ok(sfc)
    }

    /// Generate <script setup> content
    fn generate_script(&mut self, widget: &AuraWidget) -> GenResult<String> {
        let mut script = String::new();

        // Determine needed imports
        let needs_ref = !widget.state_vars.is_empty() || !self.template_refs.is_empty();
        let needs_computed = !widget.computed.is_empty();

        // Generate Vue import statement
        let mut imports = Vec::new();
        if needs_ref {
            imports.push("ref");
        }
        if needs_computed {
            imports.push("computed");
        }
        // Plan 106: Add watch, nextTick, onMounted for Prism.js re-highlighting
        if !self.previewcard_data.is_empty() {
            imports.push("watch");
            imports.push("nextTick");
            imports.push("onMounted");
        }
        // Timer/tick mechanism needs onMounted + onUnmounted
        if widget.tick_interval.is_some() {
            if !imports.contains(&"onMounted") {
                imports.push("onMounted");
            }
            imports.push("onUnmounted");
            // If there's a 'running' state var, timer is gated by watch()
            let has_running = widget.state_vars.iter().any(|s| s.name == "running");
            // If elapsed + time_display/ms_display exist, watch formats the display
            let has_elapsed = widget.state_vars.iter().any(|s| s.name == "elapsed");
            let has_time_display = widget.state_vars.iter().any(|s| s.name == "time_display");
            let has_ms_display = widget.state_vars.iter().any(|s| s.name == "ms_display");
            if has_running || (has_elapsed && (has_time_display || has_ms_display)) {
                imports.push("watch");
            }
        }
        // Dark mode: needs onMounted for system preference detection
        if self.has_dark_mode {
            if !imports.contains(&"onMounted") {
                imports.push("onMounted");
            }
        }
        // Lifecycle: .Init → onMounted, .Destroy → onUnmounted
        let has_init = widget.lifecycle.iter().any(|l| l.name == "Init");
        let has_destroy = widget.lifecycle.iter().any(|l| l.name == "Destroy");
        if has_init {
            if !imports.contains(&"onMounted") {
                imports.push("onMounted");
            }
        }
        // Auto-edit onMounted for sub-widgets with editing state + note prop
        let _has_editing = self.state_names.iter().any(|n| n == "editing");
        let _has_note_prop = self.prop_names.iter().any(|n| n == "note");
        if _has_editing && _has_note_prop {
            if !imports.contains(&"onMounted") {
                imports.push("onMounted");
            }
        }
        if has_destroy {
            if !imports.contains(&"onUnmounted") {
                imports.push("onUnmounted");
            }
        }
        // Widget-level `watch { ... }` block → Vue watch() calls
        if !widget.watchers.is_empty() && !imports.contains(&"watch") {
            imports.push("watch");
        }
        // Global (window/document-level) listeners are registered in onMounted
        // and removed in onUnmounted.
        if !self.global_listeners.is_empty() {
            if !imports.contains(&"onMounted") {
                imports.push("onMounted");
            }
            if !imports.contains(&"onUnmounted") {
                imports.push("onUnmounted");
            }
        }
        if !imports.is_empty() {
            script.push_str(&format!("import {{ {} }} from 'vue'\n", imports.join(", ")));
        }
        // Plan 106: Add Prism import for syntax highlighting
        if !self.previewcard_data.is_empty() {
            script.push_str("import Prism from 'prismjs'\n");
        }

        // Plan 105: Add router import if needed
        if self.needs_router {
            script.push_str("import { useRouter } from 'vue-router'\n");
            script.push_str("const router = useRouter()\n\n");
        }
        // Plan 235: Add useRoute import if needed
        if self.needs_route {
            script.push_str("import { useRoute } from 'vue-router'\n");
            script.push_str("const route = useRoute()\n\n");
        }

        // Generate shadcn-vue imports (if any components were used in template)
        let shadcn_imports = self.generate_shadcn_imports();
        if !shadcn_imports.is_empty() {
            script.push_str(&shadcn_imports);
            script.push('\n');
        }
        // Chart CurveType import
        if self.use_curve_type {
            script.push_str("import { CurveType } from '@unovis/ts'\n");
        }

        // Generate lucide-vue-next imports (if any icons were used)
        if !self.lucide_icons.is_empty() {
            let mut icons: Vec<String> = self.lucide_icons.iter().cloned().collect();
            icons.sort();
            script.push_str(&format!("import {{ {} }} from 'lucide-vue-next'\n", icons.join(", ")));
            script.push('\n');
        }

        // Widget `use { ... }` external imports (hand-written TS/Vue escape
        // hatch). Component imports already provided by the built-in widget
        // registry are dropped to avoid duplicate symbol bindings.
        if !self.ext_import_lines.is_empty() {
            let mut lines: Vec<String> = self
                .ext_import_lines
                .iter()
                .filter(|(names, is_component, _)| {
                    !*is_component
                        || !names.iter().any(|n| self.shadcn_components_used.contains(n))
                })
                .map(|(_, _, line)| line.clone())
                .collect();
            lines.sort();
            lines.dedup();
            for line in &lines {
                script.push_str(line);
            }
            script.push('\n');
        }

        // Widget `use { composable: ... }` — call each composable once at
        // <script setup> top level, binding the return value to a local
        // const (`useMenuBounds` → `menuBounds`) reachable from handlers.
        for (local, callee) in &self.ext_composables {
            script.push_str(&format!("const {} = {}()\n", local, callee));
        }
        if !self.ext_composables.is_empty() {
            script.push('\n');
        }

        // Import ThemeToggle custom component if used
        if self.use_theme_toggle {
            script.push_str("import ThemeToggle from '@/components/ThemeToggle.vue'\n");
        }

        // Plan 234: Import custom PascalCase components referenced in template
        // (e.g. A2UIRenderer and other embedded Vue components)
        let mut custom_imports = Vec::new();
        for comp in &self.component_refs {
            if *comp == "ThemeToggle" {
                continue; // Already handled above
            }
            // Skip shadcn components (already imported via generate_shadcn_imports)
            if self.shadcn_components_used.contains(comp) {
                continue;
            }
            custom_imports.push(format!("import {} from '@/components/{}.vue'\n", comp, comp));
        }
        if !custom_imports.is_empty() {
            custom_imports.sort();
            custom_imports.dedup();
            for imp in &custom_imports {
                script.push_str(imp);
            }
        }
        if self.use_theme_toggle || !custom_imports.is_empty() {
            script.push('\n');
        }

        // Plan 132: Scan handlers for API function calls
        if !self.explicit_api_imports {
            // Legacy mode: scan AST to discover API calls
            for (_pattern, payload) in &widget.handlers {
                self.extract_api_calls_from_payload(payload);
            }
            // Also scan lifecycle events (.Init, .Destroy) for API calls
            for lc in &widget.lifecycle {
                self.extract_api_calls_from_payload(&lc.payload);
            }
        } else {
            // Explicit import mode: collect which declared imports are actually used
            for (_pattern, payload) in &widget.handlers {
                self.extract_api_calls_from_payload(payload);
            }
            for lc in &widget.lifecycle {
                self.extract_api_calls_from_payload(&lc.payload);
            }
        }

        // Plan 132: Add API imports if needed
        if !self.api_functions_used.is_empty() {
            let api_funcs: Vec<String> = self.api_functions_used.iter().cloned().collect();
            script.push_str(&format!("import {{ {} }} from '@/lib/api'\n", api_funcs.join(", ")));
            // Deprecation warning for implicit API usage
            if !self.explicit_api_imports {
                eprintln!(
                    "  warning: Widget '{}' uses API functions [{}] without explicit import. Add `use back.api: {}` at the top of the file.",
                    self.current_widget.as_deref().unwrap_or("unknown"),
                    api_funcs.join(", "),
                    api_funcs.join(", "),
                );
            }
        }
        script.push('\n');

        // Generate state variables as ref()
        for state in &widget.state_vars {
            if !self.state_names.contains(&state.name) {
                self.state_names.push(state.name.clone());
            }
            let init = self.expr_to_js(&state.initial)?;

            // Plan 100: Add type annotation for TypeScript
            if self.use_typescript {
                let ts_type = self.expr_to_ts_type(&state.initial);
                script.push_str(&format!("const {} = ref<{}>({})\n", state.name, ts_type, init));
            } else {
                script.push_str(&format!("const {} = ref({})\n", state.name, init));
            }
        }

        if !widget.state_vars.is_empty() {
            script.push('\n');
        }

        // Template refs (`ref: "menuEl"` in the view) → Vue template-ref
        // declarations. Handlers access them via `.menuEl` → `menuEl.value!`.
        // Refs attached to child components are typed `any` (their exposed
        // surface is unknown to the parent).
        for ref_name in &self.template_refs {
            if self.component_ref_names.contains(ref_name) {
                script.push_str(&format!("const {} = ref<any>(null)\n", ref_name));
            } else {
                script.push_str(&format!("const {} = ref<HTMLElement | null>(null)\n", ref_name));
            }
        }
        if !self.template_refs.is_empty() {
            script.push('\n');
        }

        // Dark mode: detect system preference on mount (only for isDark pattern)
        if self.has_dark_mode && self.dark_mode_var.as_deref() != Some("dark_mode") {
            script.push_str("onMounted(() => {\n");
            script.push_str("  isDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches\n");
            script.push_str("})\n\n");
        }

        // Generate computed properties
        for computed_prop in &widget.computed {
            let expr_js = self.expr_to_js(&computed_prop.expr)?;

            // Plan 100: Add type annotation for TypeScript
            if self.use_typescript {
                let ts_type = self.expr_to_ts_type(&computed_prop.expr);
                script.push_str(&format!(
                    "const {} = computed<{}>(() => {})\n",
                    computed_prop.name, ts_type, expr_js
                ));
            } else {
                script.push_str(&format!(
                    "const {} = computed(() => {})\n",
                    computed_prop.name, expr_js
                ));
            }
        }

        if !widget.computed.is_empty() {
            script.push('\n');
        }

        // Generate defineProps if widget has props (sub-widget component)
        if !widget.props.is_empty() {
            // Plan 367 P1-1: collect custom type names that need importing from api.ts
            let mut custom_types: Vec<String> = Vec::new();
            script.push_str("const props = defineProps<{\n");
            for prop in &widget.props {
                // Plan 367 P1-1: map Auto types to TS types instead of using 'any'.
                let ts_type = Self::auto_type_to_ts_type(&prop.type_info);
                // Track custom types for import generation
                if let crate::ast::Type::User(decl) = &prop.type_info {
                    let name = decl.name.as_str().to_string();
                    // Only track types that aren't built-in aliases
                    if !matches!(name.as_str(), "msg" | "str" | "int" | "i64" | "uint" | "bool") {
                        if !custom_types.contains(&name) {
                            custom_types.push(name);
                        }
                    }
                }
                if prop.default.is_some() {
                    script.push_str(&format!("  {}?: {}\n", prop.name, ts_type));
                } else {
                    script.push_str(&format!("  {}: {}\n", prop.name, ts_type));
                }
            }
            script.push_str("}>()\n\n");
            // Import custom types from api.ts (type-only import)
            if !custom_types.is_empty() {
                script.push_str(&format!(
                    "import type {{ {} }} from '@/lib/api'\n\n",
                    custom_types.join(", ")
                ));
            }
        }

        // Widget-level `expose { ... }`: exposed `on` handlers must be
        // treated as used even when the template never references them — the
        // parent calls them through a template ref (defineExpose below).
        // Marked before defineEmits so their emit() payloads are declared.
        for name in &widget.exposes {
            if widget.handlers.contains_key(&format!(".{}", name)) {
                self.used_handlers.insert(name.clone());
            }
        }

        // Generate emit if needed
        if self.has_emit {
            // Build event → payload type map from msg declarations.
            // Each variant may carry a single payload type (e.g. SelectTag(str) → str).
            // handler_params stores param names (e.g. ["t"]) for the on-block's .Handler(t).
            // We use the variant's payload type when available.
            let mut event_payload_types: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            for msg in &widget.messages {
                for variant in &msg.variants {
                    // Plan 043 M5 #1: payload is now Vec<Type>; the TS event
                    // path carries a single payload type, so use the first
                    // (multi-param variants are a Rust-backend feature for now).
                    if let Some(ty) = variant.payload.first() {
                        // Only carry payload type if the handler actually has
                        // matching params (otherwise the emit() call won't pass
                        // args, causing a TS mismatch).
                        let pattern_key = format!(".{}", variant.name);
                        if Self::get_handler_params(&widget.handler_params, &pattern_key).is_some() {
                            event_payload_types.insert(variant.name.clone(), Self::auto_type_to_ts_type(ty));
                        }
                    }
                }
            }
            // Fallback: the emit() call passes handler params (see below), so a
            // handler with declared params but no msg payload type must still
            // declare matching payload arity — otherwise vue-tsc reports
            // "Expected 1 arguments, but got 2" on the emit('X', arg) call.
            for (pattern, params) in &widget.handler_params {
                if params.is_empty() {
                    continue;
                }
                let name = Self::base_pattern(pattern).trim_start_matches('.');
                if self.emit_events.contains(&name.to_string())
                    && !event_payload_types.contains_key(name)
                {
                    let any_params: Vec<&str> = params.iter().map(|_| "any").collect();
                    event_payload_types.insert(name.to_string(), any_params.join(", "));
                }
            }

            script.push_str("const emit = defineEmits<{\n");
            for event in &self.emit_events {
                // Plan 367 P1-4: only declare events for handlers that are
                // actually used in the template. Unused handlers don't get
                // function definitions, so declaring their emit types is noise.
                // used_handlers holds sanitized fn names (emit names like
                // "update:modelValue" sanitize to update_modelValue).
                if !self.used_handlers.contains(event)
                    && !self.used_handlers.contains(&Self::sanitize_ident(event))
                {
                    continue;
                }
                // TS object-literal keys must be quoted when the emit name is
                // not a plain identifier (e.g. 'update:modelValue').
                let key = if event.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$') {
                    event.clone()
                } else {
                    format!("'{}'", event)
                };
                if let Some(ts_type) = event_payload_types.get(event) {
                    script.push_str(&format!("  {}: [{}]\n", key, ts_type));
                } else {
                    script.push_str(&format!("  {}: []\n", key));
                }
            }
            script.push_str("}>()\n\n");
        }

        // Plan 351: store composable imports + const store
        if !self.store_deps.is_empty() {
            for dep in &self.store_deps {
                script.push_str(&format!(
                    "import {{ use{}Store }} from '@/stores/use{}Store'\n",
                    dep, dep
                ));
            }
            // v1: single store → const store = reactive(useXxxStore())
            // reactive() auto-unwraps nested refs so templates can use store.notes directly
            let first = &self.store_deps[0];
            script.push_str(&format!("import {{ reactive }} from 'vue'\n"));
            script.push_str(&format!("const store = reactive(use{}Store())\n\n", first));
        }

        // Widget-level `watch { ... }` block → Vue watch() calls.
        // Emitted after state/computed/defineProps so the watched refs are
        // already initialized (watch() runs immediately at setup time).
        // Source resolution: model fields and computed are refs (watched
        // directly); props need a getter (`() => props.x`).
        for watcher in &widget.watchers {
            let body = self.generate_handler_body(&watcher.payload)?;
            let sources: Vec<String> = watcher.sources.iter().map(|s| {
                if self.prop_names.contains(s) {
                    format!("() => props.{}", s)
                } else {
                    s.clone()
                }
            }).collect();
            let source = if sources.len() == 1 {
                sources[0].clone()
            } else {
                format!("[{}]", sources.join(", "))
            };
            let opts = match (watcher.immediate, watcher.deep) {
                (false, false) => String::new(),
                (immediate, deep) => {
                    let mut parts = Vec::new();
                    if immediate {
                        parts.push("immediate: true");
                    }
                    if deep {
                        parts.push("deep: true");
                    }
                    format!(", {{ {} }}", parts.join(", "))
                }
            };
            let indented = Self::indent_body(&body, "  ");
            script.push_str(&format!("watch({}, () => {{\n{}\n}}{})\n\n", source, indented, opts));
        }

        // Generate event handlers
        // fn-name → on-block base pattern (e.g. update_modelValue →
        // ".update:modelValue"): fn names are sanitized JS identifiers while
        // handler_params is keyed by the verbatim pattern, so keep the mapping.
        let mut handler_base_patterns: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for (pattern, payload) in &widget.handlers {
            let handler_name = self.pattern_to_handler_name(pattern);
            handler_base_patterns
                .insert(handler_name.clone(), Self::base_pattern(pattern).to_string());
            let mut body = self.generate_handler_body(payload)?;
            // Auto-emit events for sub-widget handlers that match emit declarations.
            // Plan 367 P0-1: Skip emit if the handler already notifies the parent
            // via a callback prop (props.on_xxx()). In that case the emit is
            // redundant — the parent's callback was already invoked.
            // Emit names are verbatim msg variant names (e.g. "update:modelValue"
            // for a v-model contract); match against the handler's fn name, which
            // is the sanitized form.
            let emit_name = if self.has_emit {
                self.emit_events
                    .iter()
                    .find(|e| e.as_str() == handler_name || Self::sanitize_ident(e) == handler_name)
                    .cloned()
            } else {
                None
            };
            if let Some(emit_name) = emit_name {
                let snake = Self::pascal_to_snake(&handler_name);
                let callback_key = format!("props.on_{}", snake);
                let already_notifies_parent = body.contains(&callback_key);
                if !already_notifies_parent {
                    // Plan 367 P1-4: pass handler params to emit() so the call
                    // matches the typed defineEmits declaration.
                    let pattern_key = Self::base_pattern(pattern);
                    let emit_args: String = Self::get_handler_params(&widget.handler_params, pattern_key)
                        .map(|params| params.iter().map(|p| p.as_str().to_string()).collect::<Vec<_>>().join(", "))
                        .unwrap_or_default();
                    if emit_args.is_empty() {
                        body.push_str(&format!("\nemit('{}')", emit_name));
                    } else {
                        body.push_str(&format!("\nemit('{}', {})", emit_name, emit_args));
                    }
                }
            }
            // Plan 132: Check if handler contains API calls (needs async)
            let is_async = self.handler_has_api_calls(payload);
            self.handlers.push((handler_name.clone(), body, is_async));
        }

        // Output handler functions
        // Plan 100: Add return type annotation for TypeScript
        // Plan 132: Add async keyword for handlers with API calls
        // Only output handlers that are actually used in the template
        let mut generated_handlers: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (handler_name, handler_body, is_async) in &self.handlers {
            // Skip unused handlers to avoid TypeScript warnings
            if !self.used_handlers.contains(handler_name) {
                continue;
            }
            generated_handlers.insert(handler_name.clone());

            // Build params: check for loop-param handlers first, then user-defined params
            // (verbatim on-block pattern — fn names are sanitized, patterns are not)
            let pattern_key = handler_base_patterns
                .get(handler_name)
                .cloned()
                .unwrap_or_else(|| format!(".{}", handler_name));
            let params_str = if let Some(loop_var) = self.loop_param_handlers.get(handler_name) {
                format!("{}: any", loop_var)
            } else {
                Self::get_handler_params(&widget.handler_params, &pattern_key)
                    .map(|params| {
                        let param_names: Vec<String> = params.iter()
                            .map(|p| format!("{}: any", p))
                            .collect();
                        param_names.join(", ")
                    })
                    .unwrap_or_default()
            };

            let async_kw = if *is_async { "async " } else { "" };
            let return_type = if self.use_typescript {
                if *is_async { ": Promise<void>" } else { ": void" }
            } else {
                ""
            };

            // For loop-param handlers with empty body, auto-generate active_id assignment
            let auto_body = if let Some(loop_var) = self.loop_param_handlers.get(handler_name) {
                if handler_body.is_empty() {
                    if let Some(target_var) = self.find_active_id_var(handler_name) {
                        Some(format!("{}.value = {}", target_var, loop_var))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(ref body) = auto_body {
                let indented = Self::indent_body(body, "  ");
                script.push_str(&format!("{}function {}({}){} {{\n{}\n}}\n\n", async_kw, handler_name, params_str, return_type, indented));
            } else if handler_body.is_empty() {
                script.push_str(&format!("{}function {}({}){} {{\n  // TODO\n}}\n\n", async_kw, handler_name, params_str, return_type));
            } else {
                let indented = Self::indent_body(handler_body, "  ");
                script.push_str(&format!("{}function {}({}){} {{\n{}\n}}\n\n", async_kw, handler_name, params_str, return_type, indented));
            }
        }

        // Generate stub functions for handlers referenced in template but not defined in on-block
        for handler_name in &self.used_handlers {
            if generated_handlers.contains(handler_name) {
                continue;
            }
            // Skip handleChildDelete — it's generated separately below
            if handler_name == "handleChildDelete" && self.needs_child_delete_handler {
                continue;
            }
            let return_type = if self.use_typescript { ": void" } else { "" };
            // Check if this stub needs loop-param
            let params_str = if let Some(loop_var) = self.loop_param_handlers.get(handler_name) {
                format!("{}: any", loop_var)
            } else {
                String::new()
            };
            let auto_body = if let Some(loop_var) = self.loop_param_handlers.get(handler_name) {
                if let Some(target_var) = self.find_active_id_var(handler_name) {
                    Some(format!("{}.value = {}", target_var, loop_var))
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(body) = auto_body {
                script.push_str(&format!("function {}({}){} {{\n  {}\n}}\n\n", handler_name, params_str, return_type, body));
            } else {
                script.push_str(&format!("function {}({}){} {{\n  // TODO: handler not defined in on-block\n}}\n\n", handler_name, params_str, return_type));
            }
        }

        // Generate handleChildDelete for parent components with array state
        // This handles the case where a sub-widget emits 'Delete' and the parent
        // needs to remove the item from its array (e.g., notes list)
        if self.needs_child_delete_handler {
            // Find the active index variable name (active_id, active_index, or active_idx)
            let active_var = self.state_names.iter()
                .find(|n| n.starts_with("active"))
                .cloned()
                .unwrap_or_else(|| "active_id".to_string());
            script.push_str("function handleChildDelete() {\n");
            script.push_str(&format!("  if ({}.value < notes.value.length) notes.value.splice({}.value, 1)\n", active_var, active_var));
            script.push_str(&format!("  if (notes.value.length > 0) {{\n"));
            script.push_str(&format!("    {}.value = 0\n", active_var));
            script.push_str("  }\n");
            if self.state_names.iter().any(|n| n == "editing") {
                script.push_str("  editing.value = false\n");
            }
            script.push_str("}\n\n");
        }

        // Generate lifecycle hooks from widget.lifecycle
        // .Init → onMounted
        if let Some(init) = widget.lifecycle.iter().find(|l| l.name == "Init") {
            let is_async = self.handler_has_api_calls(&init.payload);
            let async_kw = if is_async { "async " } else { "" };
            let body = self.generate_handler_body(&init.payload).unwrap_or_default();
            let indented = Self::indent_body(&body, "  ");
            script.push_str(&format!("onMounted({}() => {{\n{}\n}})\n\n", async_kw, indented));
        }
        // .Destroy → onUnmounted
        if let Some(destroy) = widget.lifecycle.iter().find(|l| l.name == "Destroy") {
            let body = self.generate_handler_body(&destroy.payload).unwrap_or_default();
            script.push_str(&format!("onUnmounted(() => {{\n  {}\n}})\n\n", body));
        }

        // Global (window/document-level) event listeners declared in the view
        // via `.window`/`.document` event modifiers, e.g.
        // `onmousemove.window: .DragMove($event)` or
        // `onwheel.document.capture.prevent: .LockWheel($event)`.
        // Registered on mount and removed on unmount (no leaks across
        // component instances).
        if !self.global_listeners.is_empty() {
            // Wrapper functions first (prevent/stop/arg adapters).
            for gl in &self.global_listeners {
                if let Some(wrapper) = &gl.wrapper {
                    script.push_str(wrapper);
                }
            }
            script.push_str("onMounted(() => {\n");
            for gl in &self.global_listeners {
                let options = Self::listener_options(gl, false);
                script.push_str(&format!(
                    "  {}.addEventListener('{}', {}{})\n",
                    gl.target, gl.event, gl.listener, options
                ));
            }
            script.push_str("})\n\n");
            script.push_str("onUnmounted(() => {\n");
            for gl in &self.global_listeners {
                // removeEventListener only matches on the capture flag.
                let options = Self::listener_options(gl, true);
                script.push_str(&format!(
                    "  {}.removeEventListener('{}', {}{})\n",
                    gl.target, gl.event, gl.listener, options
                ));
            }
            script.push_str("})\n\n");
        }

        // Note: auto-edit-mode onMounted was previously hardcoded here (Plan 367 P0-3
        // removed it). This logic now lives in the .at source's .Init handler,
        // which is faithfully transpiled to onMounted above. Having both caused
        // a duplicate onMounted in EditorPanel.vue.

        // Generate timer/tick mechanism (setInterval + onUnmounted cleanup)
        // The timer only runs when the widget has a `running` state var set to "true"
        if let Some(interval) = widget.tick_interval {
            // Check if there's a 'running' state variable to gate the timer
            let has_running = widget.state_vars.iter().any(|s| s.name == "running");

            if self.use_typescript {
                script.push_str("const tickTimer = ref<number | null>(null)\n\n");
            } else {
                script.push_str("const tickTimer = ref(null)\n\n");
            }

            // Find the .Tick handler body
            let tick_body = widget.handlers.get(".Tick")
                .map(|payload| self.generate_handler_body(payload).unwrap_or_default())
                .unwrap_or_default();

            if has_running {
                // Timer starts/stops based on `running` state — use watch to manage interval
                script.push_str(&format!("watch(running, (val) => {{\n  if (val === 'true' && tickTimer.value === null) {{\n    tickTimer.value = setInterval(() => {{\n      {}\n    }}, {})\n  }} else if (val !== 'true' && tickTimer.value !== null) {{\n    clearInterval(tickTimer.value)\n    tickTimer.value = null\n  }}\n}})\n\n", tick_body, interval));
            } else {
                // No running gate — start timer immediately on mount
                script.push_str(&format!("onMounted(() => {{\n  tickTimer.value = setInterval(() => {{\n    {}\n  }}, {})\n}})\n\n", tick_body, interval));
            }

            // If the widget has both `elapsed` and `time_display`/`ms_display`,
            // add a watch to format elapsed time into display strings
            let has_elapsed = widget.state_vars.iter().any(|s| s.name == "elapsed");
            let has_time_display = widget.state_vars.iter().any(|s| s.name == "time_display");
            let has_ms_display = widget.state_vars.iter().any(|s| s.name == "ms_display");
            if has_elapsed && (has_time_display || has_ms_display) {
                if !imports.contains(&"watch") {
                    imports.push("watch");
                }
                script.push_str("watch(elapsed, (ms) => {\n");
                script.push_str("  const totalSec = Math.floor(ms / 1000)\n");
                script.push_str("  const min = Math.floor(totalSec / 60)\n");
                script.push_str("  const sec = totalSec % 60\n");
                if has_time_display {
                    script.push_str("  time_display.value = String(min).padStart(2, '0') + ':' + String(sec).padStart(2, '0')\n");
                }
                if has_ms_display {
                    script.push_str("  ms_display.value = '.' + String(Math.floor((ms % 1000) / 10)).padStart(2, '0')\n");
                }
                script.push_str("})\n\n");
            }

            script.push_str("onUnmounted(() => {\n  if (tickTimer.value !== null) {\n    clearInterval(tickTimer.value)\n  }\n})\n\n");
        }

        // Generate previewcard state variables and copyCode function
        if !self.previewcard_data.is_empty() {
            // Add copiedCode state
            if self.use_typescript {
                script.push_str("const copiedCode = ref<string>('')\n");
            } else {
                script.push_str("const copiedCode = ref('')\n");
            }

            // Helper function to convert kebab-case to PascalCase
            let to_pascal_case = |s: &str| -> String {
                s.split('-')
                    .map(|part| {
                        let mut chars = part.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    })
                    .collect()
            };

            // Add state for each previewcard
            for pc in &self.previewcard_data {
                let id_pascal = to_pascal_case(&pc.id);
                let show_var = format!("show{}Code", id_pascal);
                let active_var = format!("active{}Tab", id_pascal);
                if self.use_typescript {
                    script.push_str(&format!("const {} = ref<boolean>(true)\n", show_var));  // expanded by default
                    script.push_str(&format!("const {} = ref<string>('auto')\n", active_var));
                } else {
                    script.push_str(&format!("const {} = ref(true)\n", show_var));  // expanded by default
                    script.push_str(&format!("const {} = ref('auto')\n", active_var));
                }
            }
            script.push('\n');

            // Add copyCode function
            if self.use_typescript {
                script.push_str("// Copy to clipboard function\n");
                script.push_str("async function copyCode(code: string, id: string): Promise<void> {\n");
                script.push_str("  try {\n");
                script.push_str("    await navigator.clipboard.writeText(code)\n");
                script.push_str("    copiedCode.value = id\n");
                script.push_str("    setTimeout(() => {\n");
                script.push_str("      copiedCode.value = ''\n");
                script.push_str("    }, 2000)\n");
                script.push_str("  } catch (err) {\n");
                script.push_str("    console.error('Failed to copy:', err)\n");
                script.push_str("  }\n");
                script.push_str("}\n\n");
            } else {
                script.push_str("// Copy to clipboard function\n");
                script.push_str("async function copyCode(code, id) {\n");
                script.push_str("  try {\n");
                script.push_str("    await navigator.clipboard.writeText(code)\n");
                script.push_str("    copiedCode.value = id\n");
                script.push_str("    setTimeout(() => {\n");
                script.push_str("      copiedCode.value = ''\n");
                script.push_str("    }, 2000)\n");
                script.push_str("  } catch (err) {\n");
                script.push_str("    console.error('Failed to copy:', err)\n");
                script.push_str("  }\n");
                script.push_str("}\n\n");
            }

            // Add code sample constants
            for pc in &self.previewcard_data {
                // Convert PascalCase to camelCase (e.g., "CardBasic" -> "cardBasic")
                let id_camel: String = pc.id.split('-')
                    .enumerate()
                    .map(|(i, part)| {
                        let mut chars = part.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => {
                                if i == 0 {
                                    c.to_lowercase().collect::<String>() + chars.as_str()
                                } else {
                                    c.to_uppercase().collect::<String>() + chars.as_str()
                                }
                            }
                        }
                    })
                    .collect();
                let auto_var = format!("{}AutoCode", id_camel);
                let vue_var = format!("{}VueCode", id_camel);
                script.push_str(&format!("const {} = `{}`\n", auto_var, pc.auto_code));
                script.push_str(&format!("const {} = `{}`\n", vue_var, pc.vue_code));
            }

            // Add code constants for each codeblock
            for cb in &self.codeblock_data {
                // Convert kebab-case to camelCase (e.g., "install-button" -> "installButton")
                let id_camel: String = cb.id.split('-')
                    .enumerate()
                    .map(|(i, part)| {
                        let mut chars = part.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => {
                                if i == 0 {
                                    c.to_lowercase().collect::<String>() + chars.as_str()
                                } else {
                                    c.to_uppercase().collect::<String>() + chars.as_str()
                                }
                            }
                        }
                    })
                    .collect();
                let code_var = format!("{}Code", id_camel);
                script.push_str(&format!("const {} = `{}`\n", code_var, cb.code));
            }

            // Plan 106: Add watchers for syntax highlighting when tabs change
            for pc in &self.previewcard_data {
                let id_pascal = to_pascal_case(&pc.id);
                let active_var = format!("active{}Tab", id_pascal);
                script.push_str(&format!(
                    "watch({}, () => {{\n  nextTick(() => Prism.highlightAll())\n}})\n",
                    active_var
                ));
            }

            // Add onMounted hook for initial syntax highlighting
            script.push_str("onMounted(() => {\n  nextTick(() => Prism.highlightAll())\n})\n");
            script.push('\n');
        } else if !self.codeblock_data.is_empty() {
            // Codeblocks only (no previewcard)
            // Add copiedCode state
            if self.use_typescript {
                script.push_str("const copiedCode = ref<string>('')\n");
            } else {
                script.push_str("const copiedCode = ref('')\n");
            }

            // Add copyCode function
            script.push_str("\n// Copy to clipboard function\n");
            script.push_str("async function copyCode(code: string, id: string): Promise<void> {\n");
            script.push_str("  try {\n");
            script.push_str("    await navigator.clipboard.writeText(code)\n");
            script.push_str("    copiedCode.value = id\n");
            script.push_str("    setTimeout(() => {\n");
            script.push_str("      copiedCode.value = ''\n");
            script.push_str("    }, 2000)\n");
            script.push_str("  } catch (err) {\n");
            script.push_str("    console.error('Failed to copy:', err)\n");
            script.push_str("  }\n");
            script.push_str("}\n\n");

            // Add code constants for each codeblock
            for cb in &self.codeblock_data {
                // Convert kebab-case to camelCase (e.g., "install-button" -> "installButton")
                let id_camel: String = cb.id.split('-')
                    .enumerate()
                    .map(|(i, part)| {
                        let mut chars = part.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => {
                                if i == 0 {
                                    c.to_lowercase().collect::<String>() + chars.as_str()
                                } else {
                                    c.to_uppercase().collect::<String>() + chars.as_str()
                                }
                            }
                        }
                    })
                    .collect();
                let code_var = format!("{}Code", id_camel);
                script.push_str(&format!("const {} = `{}`\n", code_var, cb.code));
            }
            script.push('\n');
        }

        // Widget-level `expose { ... }` block → defineExpose({ ... }) so a
        // parent holding a template ref on this component can call imperative
        // methods (exposed `on` handlers / imported fns) or read exposed
        // state/template refs. Vue's expose proxy unwraps refs on access, so
        // exposing the ref object directly is correct.
        if !widget.exposes.is_empty() {
            script.push_str(&format!("defineExpose({{ {} }})\n", widget.exposes.join(", ")));
        }

        Ok(script)
    }

    /// Generate handler function body from LogicPayload
    fn generate_handler_body(&self, payload: &LogicPayload) -> GenResult<String> {
        match payload {
            LogicPayload::AstStmts(stmts) => {
                let mut ctx = crate::ui_gen::ts_adapter::AuraTsContext::new(self.state_names.iter().cloned().collect())
                    .with_props(self.prop_names.iter().cloned().collect())
                    .with_refs(self.template_refs.iter().cloned().collect());
                if !self.project_api_functions.is_empty() {
                    ctx = ctx.with_api_functions(self.project_api_functions.clone());
                }
                Ok(crate::ui_gen::ts_adapter::transpile_handler_body(stmts, &ctx))
            }
            LogicPayload::Bytecode(_) => {
                Err(GenError::UnsupportedStmt("Bytecode not supported in Vue generator".to_string()))
            }
        }
    }

    /// Generate <template> content from view tree
    fn generate_template(&mut self, root: &AuraNode) -> GenResult<String> {
        let mut template = String::new();

        let root_html = self.node_to_html(root, 2)?;

        // Check for dark mode: either explicit state var, view tree reference,
        // or a ToggleDarkMode handler used in the template
        let has_toggle_dark = self.used_handlers.contains("ToggleDarkMode");
        if self.has_dark_mode || has_toggle_dark {
            let dark_expr = match &self.dark_mode_var {
                Some(var) if var == "dark_mode" => "store.dark_mode",
                _ => "isDark",
            };
            // If dark_expr is store.dark_mode but var wasn't set, use store.dark_mode
            let dark_expr = if has_toggle_dark && self.dark_mode_var.is_none() {
                "store.dark_mode"
            } else {
                dark_expr
            };
            let html = root_html.replacen("<div ", &format!("<div :class=\"{{ dark: {} }}\" ", dark_expr), 1);
            template.push_str(&html);
        } else {
            template.push_str(&root_html);
        }

        Ok(template)
    }

    /// Generate <style> content
    fn generate_style(&self) -> String {
        let mut style = String::new();

        // Plan 106: Override Prism.js default margin on pre elements
        if !self.previewcard_data.is_empty() || !self.codeblock_data.is_empty() {
            style.push_str("/* Override Prism.js default styles */\n");
            style.push_str("pre[class*=\"language-\"] {\n");
            style.push_str("  margin: 0;\n");
            style.push_str("}\n\n");
        }

        style.push_str("/* Component styles */\n");
        style
    }

    /// Dynamic component: `dyn (.item.icon) { size: 16, class: "..." }` →
    /// `<component :is="item.icon" :size="16" class="..." />`.
    ///
    /// The `is` value is an arbitrary bound expression — a for-loop iterator
    /// field (`item.icon`), a model field, or a computed. Additional props
    /// are passed through as `:prop` bindings; `class` / `style_obj` and
    /// event listeners behave like on plain elements.
    fn generate_dyn_component_html(
        &mut self,
        props: &std::collections::HashMap<String, AuraPropValue>,
        events: &std::collections::HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "  ".repeat(indent);
        let mut attrs: Vec<String> = Vec::new();

        // :is binding (required). The value may be `any` (e.g. a component
        // constructor carried in list data), so cast for vue-tsc strict mode.
        if let Some(is_value) = props.get("is") {
            if let AuraPropValue::Expr(expr) = is_value {
                let is_str = self.expr_to_vue_bound_value(expr)?;
                attrs.push(format!(":is=\"({}) as any\"", is_str));
            }
        }

        // Class attribute (both static and dynamic)
        let (static_classes, dynamic_classes) = self.extract_classes("dyn", props);
        if !static_classes.is_empty() {
            attrs.push(format!("class=\"{}\"", static_classes));
        }
        if let Some(dynamic) = dynamic_classes {
            attrs.push(format!(":class=\"{}\"", dynamic));
        }

        // Remaining props as v-bind attributes
        for (key, value) in props {
            if key == "is" || key == "class" || key == "style" {
                continue;
            }
            // Inline style object: style_obj: { top: expr } → :style="{...}"
            if key == "style_obj" {
                if let AuraPropValue::StyleBinding(bindings) = value {
                    attrs.push(format!(":style=\"{}\"", self.style_obj_to_vue(bindings)));
                }
                continue;
            }
            match value {
                AuraPropValue::Expr(expr) => {
                    let value_str = self.expr_to_vue_bound_value(expr)?;
                    attrs.push(format!(":{}=\"{}\"", key, value_str));
                }
                AuraPropValue::StyleBinding(_) => {}
            }
        }

        // Event listeners (same conventions as plain elements)
        for (event, aura_event) in events {
            // .window/.document modifiers → global listener, no template attr
            if self.try_register_global_listener(event, aura_event) {
                continue;
            }
            let vue_event = self.auto_event_to_vue(event);
            let mut handler_fn = self.handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
            let handler_name = self.handler_to_function_call(&aura_event.handler);
            // Inside a for-loop, pass the loop variable when the handler
            // doesn't already have explicit params.
            if let Some(ref loop_var) = self.current_loop_var {
                if aura_event.params.is_empty() {
                    handler_fn = format!("{}({})", handler_fn, loop_var);
                    self.loop_param_handlers.insert(handler_name.clone(), loop_var.clone());
                }
            }
            self.used_handlers.insert(handler_name);
            attrs.push(format!("{}=\"{}\"", vue_event, handler_fn));
        }

        let attr_str = if attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", attrs.join(" "))
        };

        if children.is_empty() {
            Ok(format!("{}<component{} />\n", ind, attr_str))
        } else {
            let mut html = format!("{}<component{}>\n", ind, attr_str);
            for child in children {
                // Component children: slot(name:) elements target named slots.
                html.push_str(&self.slot_child_to_html(child, indent + 1)?);
            }
            html.push_str(&format!("{}</component>\n", ind));
            Ok(html)
        }
    }

    /// Generate a slot outlet: `slot` → `<slot />`,
    /// `slot(name: "header")` → `<slot name="header" />`.
    /// Children (if any) become Vue slot fallback content.
    fn generate_slot_outlet_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "  ".repeat(indent);
        let name_attr = props
            .get("name")
            .and_then(|v| self.extract_string_value(v))
            .map(|n| format!(" name=\"{}\"", n))
            .unwrap_or_default();
        if children.is_empty() {
            Ok(format!("{}<slot{} />\n", ind, name_attr))
        } else {
            let mut html = format!("{}<slot{}>\n", ind, name_attr);
            for child in children {
                html.push_str(&self.node_to_html(child, indent + 1)?);
            }
            html.push_str(&format!("{}</slot>\n", ind));
            Ok(html)
        }
    }

    /// Emit a child of a component instantiation (sub-widget, external
    /// `use { component }`, dyn `<component :is>`, or PascalCase component).
    /// A `slot(name: "x") { ... }` element in this position targets the
    /// child's named slot → `<template #x>...</template>`; a bare
    /// `slot { ... }` unwraps its children into the default slot.
    /// Anything else is emitted normally.
    fn slot_child_to_html(&mut self, child: &AuraNode, indent: usize) -> GenResult<String> {
        if let AuraNode::Element { tag, props, children, .. } = child {
            if tag == "slot" || tag == "Slot" {
                let ind = "  ".repeat(indent);
                if let Some(name) = props.get("name").and_then(|v| self.extract_string_value(v)) {
                    let name = name.to_string();
                    let mut html = format!("{}<template #{}>\n", ind, name);
                    for c in children {
                        html.push_str(&self.node_to_html(c, indent + 1)?);
                    }
                    html.push_str(&format!("{}</template>\n", ind));
                    return Ok(html);
                }
                let mut html = String::new();
                for c in children {
                    html.push_str(&self.node_to_html(c, indent)?);
                }
                return Ok(html);
            }
        }
        self.node_to_html(child, indent)
    }

    /// Emit an `if`/`else if`/`else` chain as flat sibling `<template>` nodes.
    ///
    /// Plan 043 M5 #3 — the parser (`parse_view_conditional`) nests an
    /// `else if` chain as `else_body = Some([Conditional(...)])`, so a chain
    /// `if A {} else if B {} else if C {} else {D}` becomes
    /// `A → else[B → else[C → else[D]]]`. This helper walks that nesting and
    /// emits one `<template>` per arm at the *same* indent (Vue requires the
    /// chain arms to be contiguous siblings with no nodes between them).
    ///
    /// `is_continuation` distinguishes the head (`v-if`) from arms deeper in
    /// the chain (`v-else-if`); a plain `else` arm closes the chain.
    fn emit_conditional(&mut self, node: &AuraNode, indent: usize, is_continuation: bool) -> GenResult<String> {
        let AuraNode::Conditional { condition, then_body, else_body, .. } = node else {
            // Not a Conditional — fall back to ordinary rendering.
            return self.node_to_html(node, indent);
        };
        let ind = "  ".repeat(indent);

        let head_attr = if is_continuation {
            format!("v-else-if=\"{}\"", self.convert_condition(condition))
        } else {
            format!("v-if=\"{}\"", self.convert_condition(condition))
        };

        let mut then_html = String::new();
        for child in then_body {
            then_html.push_str(&self.node_to_html(child, indent + 1)?);
        }

        // Tail of the chain. A single nested Conditional is a continuation
        // (another v-else-if, or a final v-else) — recurse at the SAME indent
        // so the arm stays a sibling, not a child.
        let tail = match else_body {
            Some(nodes) if nodes.len() == 1 && matches!(nodes[0], AuraNode::Conditional { .. }) => {
                self.emit_conditional(&nodes[0], indent, true)?
            }
            Some(nodes) => {
                let mut else_html = String::new();
                for child in nodes {
                    else_html.push_str(&self.node_to_html(child, indent + 1)?);
                }
                format!("{}<template v-else>\n{}{}</template>\n", ind, else_html, ind)
            }
            None => String::new(),
        };

        Ok(format!(
            "{}<template {}>\n{}{}</template>\n{}",
            ind, head_attr, then_html, ind, tail
        ))
    }

    /// Convert AuraNode to HTML string
    fn node_to_html(&mut self, node: &AuraNode, indent: usize) -> GenResult<String> {
        let ind = "  ".repeat(indent);

        match node {
            AuraNode::Element { tag, props, events, children, .. } => {
                // Special handling for previewcard element (supports both previewcard and preview-card)
                if tag == "previewcard" || tag == "preview-card" {
                    return self.generate_previewcard_html(props, events, children, indent);
                }

                // Special handling for codeblock element (with copy button)
                if tag == "codeblock" || tag == "code-block" {
                    return self.generate_codeblock_html(props, events, children, indent);
                }

                // Special handling for icon element - render as Lucide Vue component
                if tag == "icon" || tag == "Icon" {
                    let icon_name = props.get("name")
                        .and_then(|v| self.extract_string_value(v))
                        .unwrap_or("circle");
                    let lucide_component = Self::kebab_to_pascal(icon_name);
                    self.lucide_icons.insert(lucide_component.clone());

                    let (static_classes, _dynamic_classes) = self.extract_classes(tag, props);
                    let class_str = if static_classes.is_empty() {
                        String::new()
                    } else {
                        format!(" class=\"{}\"", static_classes)
                    };

                    if children.is_empty() {
                        return Ok(format!("{}<{}{} />\n", ind, lucide_component, class_str));
                    } else {
                        let mut html = format!("{}<{}{}>\n", ind, lucide_component, class_str);
                        for child in children {
                            html.push_str(&self.node_to_html(child, indent + 1)?);
                        }
                        html.push_str(&format!("{}</{}>\n", ind, lucide_component));
                        return Ok(html);
                    }
                }

                // Special handling for dyn element — dynamic component:
                // dyn (.item.icon) { size: 16, class: "..." } →
                // <component :is="item.icon" :size="16" class="..." />
                if tag == "dyn" {
                    return self.generate_dyn_component_html(props, events, children, indent);
                }

                // Slot outlet (Plan: slots): `slot` → <slot />,
                // `slot(name: "header")` → <slot name="header" />.
                // Children (if any) become Vue slot fallback content.
                // Must run before the unknown-tag div fallback in map_tag.
                // (Named-slot TARGETING at the parent side — slot(name:) used
                // as a component child — is handled by slot_child_to_html at
                // the component children emission sites below.)
                if tag == "slot" || tag == "Slot" {
                    return self.generate_slot_outlet_html(props, children, indent);
                }

                // Special handling for category-section element
                if tag == "category-section" || tag == "category_section" {
                    return self.generate_category_section_html(props, children, indent);
                }

                // Check if this is a known sub-widget (custom component, not shadcn)
                let is_known_sub_widget = self.known_sub_widgets.contains(tag);

                // Check if this is an external component declared via the
                // widget `use { component: ... }` block — bound generically
                // like a sub-widget (all props v-bind, events as @emit).
                let is_external_component = self.ext_components.contains_key(tag);

                // Check if this is a shadcn-vue component
                // Note: We need to check both the original tag and lowercase version because registry uses lowercase keys
                let tag_lower = tag.to_lowercase();
                // If user provides a class prop on form elements, force native HTML
                // (e.g., TodoMVC needs <input type="checkbox" class="toggle"> not <Checkbox>)
                let has_user_class = props.contains_key("class") || props.contains_key("style");
                let force_native_elements = ["checkbox", "input", "button", "textarea"];
                let force_native = has_user_class && force_native_elements.contains(&tag_lower.as_str());

                // Determine HTML tag: when force_native, use plain HTML; otherwise map_tag handles shadcn
                let html_tag = if force_native {
                    match tag_lower.as_str() {
                        "checkbox" => "input".to_string(),
                        _ => tag_lower.clone(),
                    }
                } else {
                    self.map_tag(tag, children.is_empty())
                };
                // DBG: disabled to avoid log spam
                // eprintln!("DBG shadcn: tag={} is_sub={} force_nat={} is_shadcn={} reg_vue={} reg_vue_lower={}",
                //     tag, is_known_sub_widget, force_native, self.is_shadcn(),
                //     self.widget_registry.is_backend_supported("vue", tag),
                //     self.widget_registry.is_backend_supported("vue", &tag_lower));
                let is_shadcn_component = !is_known_sub_widget && !is_external_component && !force_native && self.is_shadcn() &&
                    (self.widget_registry.is_backend_supported("vue", tag) ||
                     self.widget_registry.is_backend_supported("vue", &tag_lower));

                // For known sub-widgets and external (widget `use`) components,
                // use component-style prop passing
                if is_known_sub_widget || is_external_component {
                    let mut attrs = Vec::new();
                    // Track first prop expression for :key binding
                    let mut first_prop_expr: Option<String> = None;
                    // Pass all props as v-bind (:prop="expr" needs a JS expression, not template text)
                    for (key, value) in props {
                        // Template ref on a child component: `ref: "canvasRef"`
                        // → static `ref="canvasRef"` attribute + a `ref<any>`
                        // declaration in <script setup> (the child's
                        // defineExpose surface is unknown to the parent).
                        // Lets handlers call exposed methods via `.canvasRef`.
                        if key == "ref" {
                            let ref_name: String = match value {
                                AuraPropValue::Expr(crate::ast::Expr::Str(name)) => name.to_string(),
                                AuraPropValue::Expr(crate::ast::Expr::Ident(name)) => name.to_string(),
                                _ => continue,
                            };
                            if !ref_name.is_empty() {
                                if !self.template_refs.contains(&ref_name) {
                                    self.template_refs.push(ref_name.clone());
                                }
                                self.component_ref_names.insert(ref_name.clone());
                                attrs.push(format!("ref=\"{}\"", ref_name));
                            }
                            continue;
                        }
                        // Inline style object on a component: style_obj → :style="{...}"
                        if key == "style_obj" {
                            if let AuraPropValue::StyleBinding(bindings) = value {
                                attrs.push(format!(":style=\"{}\"", self.style_obj_to_vue(bindings)));
                            }
                            continue;
                        }
                        let value_str = match value {
                            AuraPropValue::Expr(expr) => self.expr_to_vue_bound_value(expr)?,
                            AuraPropValue::StyleBinding(_) => "\"\"".to_string(),
                        };
                        if first_prop_expr.is_none() {
                            first_prop_expr = Some(value_str.clone());
                        }
                        attrs.push(format!(":{}=\"{}\"", key, value_str));
                    }
                    // Add :key binding for component reuse identity.
                    //
                    // We use a per-widget-instance counter so each component usage site
                    // gets a unique, STABLE key (e.g. 'AutoDownEditor-0', 'AutoDownEditor-1').
                    // This is important because:
                    //   1. Two components with the same name in different v-if branches
                    //      must NOT share a key — otherwise Vue patches in place instead
                    //      of mounting a fresh instance (breaks Tiptap editor init).
                    //   2. The key must stay stable across prop updates so components
                    //      are reused (not destroyed/recreated) when only props change
                    //      (prevents Tiptap unmount errors on note switch).
                    //   3. Inside a for-loop, items need per-item keys for correct diff.
                    //
                    // Explicit override: a `key:` prop on the instantiation
                    // (e.g. `EditorTab(key: tab.path, ...)`) is emitted above as
                    // `:key="tab.path"` and WINS — no auto-key is added (a second
                    // :key would be a duplicate attribute).
                    let has_explicit_key = props.contains_key("key");
                    self.widget_key_counter += 1;
                    if has_explicit_key {
                        // explicit :key already emitted with the other props
                    } else if let Some(ref loop_var) = self.current_loop_var {
                        if self.current_loop_var_is_index {
                            // Index var is a primitive int — `i?.id` is meaningless;
                            // use the index itself as the per-item key.
                            attrs.push(format!(":key=\"'{}-{}-' + {}\"", html_tag, self.widget_key_counter, loop_var));
                        } else {
                            attrs.push(format!(":key=\"'{}-{}-' + ({}?.id ?? {})\"", html_tag, self.widget_key_counter, loop_var, loop_var));
                        }
                    } else if let Some(ref expr) = first_prop_expr {
                        // Non-loop component: if the first prop looks like an object
                        // reference (contains '[' for index access, suggesting an array
                        // element like store.notes[idx]), bind key to its .id so the
                        // component REMOUNTS when the underlying object changes.
                        // Skip for primitive props (search: str, active_id: int) — those
                        // don't have .id and would cause TS errors.
                        if expr.contains('[') {
                            attrs.push(format!(":key=\"'{}-{}-' + ({}?.id ?? 'new')\"", html_tag, self.widget_key_counter, expr));
                        } else {
                            attrs.push(format!(":key=\"'{}-{}'\"", html_tag, self.widget_key_counter));
                        }
                    } else {
                        attrs.push(format!(":key=\"'{}-{}'\"", html_tag, self.widget_key_counter));
                    }
                    // Event handlers
                    for (event, aura_event) in events {
                        // .window/.document modifiers → global listener, no template attr
                        if self.try_register_global_listener(event, aura_event) {
                            continue;
                        }
                        let vue_event = self.auto_event_to_vue(event);
                        let mut handler_fn = self.handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
                        let handler_name = self.handler_to_function_call(&aura_event.handler);
                        // If inside a for-loop, pass the loop variable's .id as argument
                        // Only append if handler doesn't already have params from aura_event
                        if let Some(ref loop_var) = self.current_loop_var {
                            if aura_event.params.is_empty() {
                                handler_fn = format!("{}({})", handler_fn, loop_var);
                                // Plan 345: only register as a loop-param handler when we
                                // actually auto-pass the loop var. A handler with explicit
                                // args (e.g. .SelectNote(note.id)) must keep its declared
                                // param name, not be renamed to the loop variable.
                                self.loop_param_handlers.insert(handler_name.clone(), loop_var.clone());
                            }
                        }
                        self.used_handlers.insert(handler_name);
                        attrs.push(format!("{}=\"{}\"", vue_event, handler_fn));
                    }
                    // Auto-wire child Delete event to parent handler
                    // If parent has an array state (e.g., 'notes'), generate @delete handler
                    if self.state_names.iter().any(|n| n == "notes") {
                        attrs.push("@delete=\"handleChildDelete\"".to_string());
                        self.used_handlers.insert("handleChildDelete".to_string());
                        self.needs_child_delete_handler = true;
                    }
                    let attr_str = if attrs.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", attrs.join(" "))
                    };
                    // Component with children
                    if children.is_empty() {
                        return Ok(format!("{}<{}{} />\n", ind, html_tag, attr_str));
                    } else {
                        let mut html = format!("{}<{}{}>\n", ind, html_tag, attr_str);
                        for child in children {
                            // Component children: slot(name:) elements target named slots.
                            html.push_str(&self.slot_child_to_html(child, indent + 1)?);
                        }
                        html.push_str(&format!("{}</{}>\n", ind, html_tag));
                        return Ok(html);
                    }
                }

                // Build attributes
                let (attrs, text_content, generated_children) = if is_shadcn_component {
                    // Use shadcn-specific attribute generation (includes event handling)
                    let (shadcn_attrs, mut slot_content, slot_children) = self.generate_shadcn_attrs(tag, props, events);
                    // Plan 354: if no slot content from props, check primary positional text.
                    // The view parser puts positional text (e.g. `badge t`) in props as "text"
                    // OR as a text child node. Check both.
                    if slot_content.is_none() {
                        // Check if "text" prop exists (positional text stored as named prop)
                        if let Some(value) = props.get("text") {
                            slot_content = self.prop_to_text_content(value).ok();
                        }
                        // Also check children for a text node
                        if slot_content.is_none() {
                            for child in children {
                                if let AuraNode::Text(content) = child {
                                    match content {
                                        AuraTextContent::Literal(s) => {
                                            slot_content = Some(s.clone());
                                            break;
                                        }
                                        AuraTextContent::Interpolated { template, bindings } => {
                                            // Convert to Vue text
                                            let mut vue_text = template.clone();
                                            for binding in bindings {
                                                vue_text = vue_text.replace(
                                                    &format!("${{{}.{}}}", ".", binding),
                                                    &format!("{{{{ {} }}}}", binding)
                                                );
                                                vue_text = vue_text.replace(
                                                    &format!("${{{}}}", binding),
                                                    &format!("{{{{ {} }}}}", binding)
                                                );
                                            }
                                            slot_content = Some(vue_text);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    (shadcn_attrs, slot_content, slot_children)
                } else {
                    // Use plain Tailwind attribute generation
                    let mut attrs = Vec::new();
                    let mut text_content: Option<String> = None;

                    // Class attribute (both static and dynamic)
                    let (static_classes, dynamic_classes) = self.extract_classes(tag, props);
                    if !static_classes.is_empty() {
                        attrs.push(format!("class=\"{}\"", static_classes));
                    }
                    if let Some(dynamic) = dynamic_classes {
                        attrs.push(format!(":class=\"{}\"", dynamic));
                    }

                    // Auto-add type attribute for checkbox (native HTML needs type="checkbox")
                    let tag_lower_for_type = tag.to_lowercase();
                    if tag_lower_for_type == "checkbox" {
                        attrs.push("type=\"checkbox\"".to_string());
                    }

                    // Track value state ref for v-model optimization
                    // When input has both :value="stateRef" and @input handler,
                    // use v-model instead (native HTML two-way binding)
                    let mut value_state_ref: Option<String> = None;

                    // Props as attributes
                    for (key, value) in props {
                        if key == "class" || key == "style" {
                            continue; // Already handled in extract_classes
                        }
                        // Inline style object: style_obj: { top: f"${.y}px", "z-index": 50 }
                        // → :style="{ top: `${y}px`, 'z-index': 50 }"
                        if key == "style_obj" {
                            if let AuraPropValue::StyleBinding(bindings) = value {
                                attrs.push(format!(":style=\"{}\"", self.style_obj_to_vue(bindings)));
                            }
                            continue;
                        }
                        // Template ref: `ref: "menuEl"` → static `ref="menuEl"`
                        // attribute + a `const menuEl = ref<HTMLElement | null>(null)`
                        // declaration in <script setup> (emitted later from
                        // self.template_refs). Accessible in `on` handlers as
                        // `.menuEl` (→ `menuEl.value!`). When the tag is a Vue
                        // component (PascalCase), the ref is typed `any` — the
                        // child's defineExpose surface is unknown here.
                        if key == "ref" {
                            let ref_name: String = match value {
                                AuraPropValue::Expr(crate::ast::Expr::Str(name)) => name.to_string(),
                                AuraPropValue::Expr(crate::ast::Expr::Ident(name)) => name.to_string(),
                                _ => continue,
                            };
                            if !ref_name.is_empty() && !self.template_refs.contains(&ref_name) {
                                self.template_refs.push(ref_name.clone());
                            }
                            if html_tag.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                                self.component_ref_names.insert(ref_name.clone());
                            }
                            attrs.push(format!("ref=\"{}\"", ref_name));
                            continue;
                        }
                        if key == "gap" {
                            continue; // Handled in extract_classes for layout elements
                        }
                        if key == "text" {
                            text_content = Some(self.prop_to_text_content(value)?);
                            continue;
                        }
                        // Special handling for codeblock's code prop - render as content
                        if key == "code" && (tag == "codeblock" || tag == "code-block") {
                            text_content = Some(self.prop_to_text_content(value)?);
                            continue;
                        }

                        // Checkbox: native <input type="checkbox"> uses :checked, not :model-value
                        if tag == "checkbox" && key == "checked" {
                            if let Some(model) = self.extract_state_ref(value) {
                                attrs.push(format!(":checked=\"{}\"", model));
                            } else if let AuraPropValue::Expr(expr) = value {
                                if let Ok(js_expr) = self.expr_to_vue_bound_value(expr) {
                                    attrs.push(format!(":checked=\"{}\"", js_expr));
                                }
                            }
                            continue;
                        }

                        // Use v-bind (:attr) for dynamic values, static quotes for literals
                        if let AuraPropValue::Expr(crate::ast::Expr::Ident(name)) = value {
                            // Track value state ref for v-model optimization on input elements
                            if key == "value" && (tag == "input" || tag == "textarea") {
                                value_state_ref = Some(name.to_string());
                            }
                            attrs.push(format!(":{}=\"{}\"", key, name));
                        } else if let AuraPropValue::Expr(expr) = value {
                            // Plan 351: ALL expression prop values (FieldAccess,
                            // Index, etc.) use v-bind with bound JS value (no {{ }}).
                            let value_str = self.expr_to_vue_bound_value(expr)?;
                            // Also track value ref for v-model optimization
                            if key == "value" && (tag == "input" || tag == "textarea") {
                                // Handle both Expr::Ident(".xxx") and Expr::Dot(Ident("self"), "xxx")
                                match expr {
                                    crate::ast::Expr::Ident(name) => {
                                        let resolved = if name.starts_with('.') { &name[1..] } else { name.as_str() };
                                        value_state_ref = Some(resolved.to_string());
                                    }
                                    crate::ast::Expr::Dot(obj, field) => {
                                        if let crate::ast::Expr::Ident(obj_name) = obj.as_ref() {
                                            if obj_name == "self" {
                                                value_state_ref = Some(field.to_string());
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            attrs.push(format!(":{}=\"{}\"", key, value_str));
                        } else {
                            let value_str = self.prop_to_attr_value(value)?;
                            attrs.push(format!("{}={}", key, value_str));
                        }
                    }

                    // Event handlers
                    for (event, aura_event) in events {
                        // .window/.document modifiers → global listener, no template attr
                        if self.try_register_global_listener(event, aura_event) {
                            continue;
                        }
                        // v-model optimization: when input/textarea has both :value="stateRef"
                        // and @input handler, replace with v-model (native HTML two-way binding)
                        if (event == "oninput" || event == "onInput") && value_state_ref.is_some() {
                            // Replace the :value binding with v-model
                            let model_ref = value_state_ref.as_ref().unwrap();
                            // Remove the existing :value attribute and add v-model instead
                            if let Some(pos) = attrs.iter().position(|a| a.starts_with(":value=\"")) {
                                attrs[pos] = format!("v-model=\"{}\"", model_ref);
                            } else {
                                attrs.push(format!("v-model=\"{}\"", model_ref));
                            }
                            // Track the handler but don't emit @input event (v-model handles it)
                            let handler_name = self.handler_to_function_call(&aura_event.handler);
                            self.used_handlers.insert(handler_name);
                            continue;
                        }
                        let vue_event = self.auto_event_to_vue(event);
                        let mut handler_fn = self.handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
                        // Track used handler (without params for matching)
                        let handler_name = self.handler_to_function_call(&aura_event.handler);
                        // If inside a for-loop and the handler doesn't already have params,
                        // pass the loop variable's .id as argument (e.g., SelectNote(note.id))
                        if let Some(ref loop_var) = self.current_loop_var {
                            if aura_event.params.is_empty() {
                                handler_fn = format!("{}({})", handler_fn, loop_var);
                                // Plan 345: only register as a loop-param handler when we
                                // actually auto-pass the loop var. A handler with explicit
                                // args (e.g. .SelectNote(note.id)) must keep its declared
                                // param name, not be renamed to the loop variable.
                                self.loop_param_handlers.insert(handler_name.clone(), loop_var.clone());
                            }
                        }
                        self.used_handlers.insert(handler_name);
                        attrs.push(format!("{}=\"{}\"", vue_event, handler_fn));
                    }

                    (attrs, text_content, None)
                };

                let attr_str = if attrs.is_empty() {
                    String::new()
                } else {
                    format!(" {}", attrs.join(" "))
                };

                // Plan 360: For custom Vue components (PascalCase tag like AutoDownEditor,
                // NavTree, EditorPanel, etc.), add a stable unique :key so that two
                // components with the same name in different v-if branches don't collide.
                // Without this, Vue patches in place and components that rely on fresh
                // mount semantics (e.g. Tiptap editor) fail to initialize.
                // The key is stable across re-renders (counter-based, reset per SFC) so
                // components are reused when only props change.
                let is_vue_component = html_tag.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                let attr_str = if is_vue_component && !attr_str.contains(":key=") {
                    self.widget_key_counter += 1;
                    if let Some(ref loop_var) = self.current_loop_var {
                        if self.current_loop_var_is_index {
                            // Index var is a primitive int — `i?.id` is meaningless;
                            // use the index itself as the per-item key.
                            format!("{} :key=\"'{}-{}-' + {}\"", attr_str, html_tag, self.widget_key_counter, loop_var)
                        } else {
                            format!("{} :key=\"'{}-{}-' + ({}?.id ?? {})\"", attr_str, html_tag, self.widget_key_counter, loop_var, loop_var)
                        }
                    } else {
                        format!("{} :key=\"'{}-{}'\"", attr_str, html_tag, self.widget_key_counter)
                    }
                } else {
                    attr_str
                };

                // Check if we have text content (render as inline content)
                if let Some(text) = &text_content {
                    if children.is_empty() {
                        // <button @click="handler">text</button>
                        Ok(format!("{}<{}{}>{}</{}>\n", ind, html_tag, attr_str, text, html_tag))
                    } else {
                        // Has both text and children - unusual but handle it
                        let mut html = format!("{}<{}{}>{}\n", ind, html_tag, attr_str, text);
                        for child in children {
                            // Component children: slot(name:) targets named slots.
                            if is_vue_component {
                                html.push_str(&self.slot_child_to_html(child, indent + 1)?);
                            } else {
                                html.push_str(&self.node_to_html(child, indent + 1)?);
                            }
                        }
                        html.push_str(&format!("{}</{}>\n", ind, html_tag));
                        Ok(html)
                    }
                } else if children.is_empty() && generated_children.is_none() {
                    // No children and no generated children - self-closing tag
                    Ok(format!("{}<{}{} />\n", ind, html_tag, attr_str))
                } else {
                    // Has children (from source or generated)
                    let mut html = format!("{}<{}{}>\n", ind, html_tag, attr_str);

                    // Add generated children first (e.g., AvatarImage, AvatarFallback)
                    if let Some(gen_children) = &generated_children {
                        html.push_str(&format!("{}{}", "  ".repeat(indent + 1), gen_children));
                    }

                    // Add source children
                    for child in children {
                        // Component children: slot(name:) targets named slots.
                        if is_vue_component {
                            html.push_str(&self.slot_child_to_html(child, indent + 1)?);
                        } else {
                            html.push_str(&self.node_to_html(child, indent + 1)?);
                        }
                    }
                    html.push_str(&format!("{}</{}>\n", ind, html_tag));
                    Ok(html)
                }
            }

            AuraNode::Text(content) => {
                match content {
                    AuraTextContent::Literal(s) => {
                        Ok(format!("{}{}\n", ind, s))
                    }
                    AuraTextContent::Interpolated { template, bindings } => {
                        // Convert template to Vue interpolation
                        let mut vue_text = template.clone();
                        for binding in bindings {
                            // Replace ${.binding} with {{ binding }} (state reference)
                            vue_text = vue_text.replace(
                                &format!("${{{}.{}}}", ".", binding),
                                &format!("{{{{ {} }}}}", binding)
                            );
                            // Replace ${binding} with {{ binding }} (variable reference)
                            vue_text = vue_text.replace(
                                &format!("${{{}}}", binding),
                                &format!("{{{{ {} }}}}", binding)
                            );
                            // Also handle $binding format (without braces)
                            vue_text = vue_text.replace(
                                &format!("${}", binding),
                                &format!("{{{{ {} }}}}", binding)
                            );
                        }
                        // Plan 351: strip `self.` prefix from Vue interpolations.
                        // The view parser emits `self.xxx` for implicit-dot field access,
                        // but Vue <script setup> has no `self` — props/state are bare names.
                        vue_text = vue_text.replace("{{ self.", "{{ ");
                        vue_text = vue_text.replace(".self.", ".");
                        Ok(format!("{}{}\n", ind, vue_text))
                    }
                }
            }

            AuraNode::ForLoop { var, index, iterable, body, .. } => {
                // Generate v-for directive
                // Auto syntax: for idx, item in list (index first, value second)
                // Vue syntax: v-for="(item, index) in list" (value first, index second)
                // So we need to swap the order for Vue
                let iterable_name = iterable.trim_start_matches('.');
                // Auto-add search filter when widget has a 'search' state and iterates over an array
                let v_for_iterable = if self.state_names.iter().any(|n| n == "search")
                    && self.state_names.iter().any(|n| n == iterable_name) {
                    format!("{}.filter((n: any) => !search || n.title?.toLowerCase().includes(search.toLowerCase()))", iterable_name)
                } else {
                    iterable_name.to_string()
                };
                let v_for = if let Some(idx) = index {
                    format!("v-for=\"({}, {}) in {}\"", var, idx, v_for_iterable)
                } else {
                    format!("v-for=\"{} in {}\"", var, v_for_iterable)
                };

                // Set loop variable context so child events can pass it as arg.
                // Plan 346: When the loop has an index (for i, note in ...), use
                // the INDEX variable (i) as the loop_var — handlers like
                // SelectNote(i) pass the index, not the value.
                let prev_loop_var = self.current_loop_var.clone();
                let prev_loop_var_is_index = self.current_loop_var_is_index;
                self.current_loop_var = Some(index.clone().unwrap_or_else(|| var.clone()));
                self.current_loop_var_is_index = index.is_some();

                // If body has a single Element or Component, put v-for directly on it
                // to avoid <template> scoping issues with vue-tsc
                let result = if body.len() == 1 {
                    match &body[0] {
                        AuraNode::Element { .. } | AuraNode::Component { .. } => {
                            let child_html = self.node_to_html(&body[0], indent)?;
                            if let Some(gt_pos) = child_html.find('>') {
                                let mut result = child_html;
                                // Self-closing tag (<Foo />): insert before the
                                // '/', not between '/' and '>'.
                                let insert_pos = if gt_pos > 0 && result.as_bytes()[gt_pos - 1] == b'/' {
                                    gt_pos - 1
                                } else {
                                    gt_pos
                                };
                                result.insert_str(insert_pos, &format!(" {}", v_for));
                                Some(Ok(result))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                };

                if let Some(r) = result {
                    self.current_loop_var = prev_loop_var;
                    self.current_loop_var_is_index = prev_loop_var_is_index;
                    return r;
                }

                // Fallback: wrap body in a div with v-for
                // Keep current_loop_var set while processing children so
                // event handlers inside the loop get proper loop-param tracking
                let mut body_html = String::new();
                for child in body {
                    body_html.push_str(&self.node_to_html(child, indent + 1)?);
                }
                self.current_loop_var = prev_loop_var;
                self.current_loop_var_is_index = prev_loop_var_is_index;
                Ok(format!("{}<div {}>\n{}{}</div>\n", ind, v_for, body_html, ind))
            }

            AuraNode::Conditional { .. } => {
                // Plan 043 M5 #3: delegate to the recursive helper that flattens
                // if/else-if/else chains into sibling <template> nodes (v-if /
                // v-else-if / v-else). Previously this branch re-entered itself
                // via node_to_html for chain continuations, which unconditionally
                // re-emitted a top-level <template v-if> and produced wrongly
                // nested <template v-else><template v-if>.
                self.emit_conditional(node, indent, false)
            }

            AuraNode::Component { name, props, events, .. } => {
                // Build props as bindings
                let mut attrs = Vec::new();
                for (key, value) in props {
                    // Template ref escape hatch on a child component:
                    // `ref: "canvasRef"` → static `ref="canvasRef"` attribute
                    // + a script-setup ref declaration (typed `any` — the
                    // child's defineExpose surface is unknown here). Lets the
                    // parent call exposed methods via `.canvasRef.method()`.
                    if key == "ref" {
                        let ref_name = match value {
                            crate::ast::Expr::Str(n) | crate::ast::Expr::Ident(n) => n.to_string(),
                            _ => String::new(),
                        };
                        if !ref_name.is_empty() {
                            if !self.template_refs.contains(&ref_name) {
                                self.template_refs.push(ref_name.clone());
                            }
                            self.component_ref_names.insert(ref_name.clone());
                            attrs.push(format!("ref=\"{}\"", ref_name));
                        }
                        continue;
                    }
                    let value_str = self.prop_to_attr_value(&AuraPropValue::Expr(value.clone()))?;
                    attrs.push(format!(":{}={}", key, value_str));
                }

                // Event handlers
                for (event, aura_event) in events {
                    let vue_event = self.auto_event_to_vue(event);
                    let handler_fn = self.handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
                    // Track used handler (without params for matching)
                    let handler_name = self.handler_to_function_call(&aura_event.handler);
                    self.used_handlers.insert(handler_name);
                    attrs.push(format!("{}=\"{}\"", vue_event, handler_fn));
                }

                let attr_str = if attrs.is_empty() {
                    String::new()
                } else {
                    format!(" {}", attrs.join(" "))
                };

                self.component_refs.push(name.clone());
                Ok(format!("{}<{}{} />\n", ind, name, attr_str))
            }

            // Plan 105: Router outlet and link
            AuraNode::Outlet => {
                // Vue Router outlet: <router-view />
                self.needs_router = true;
                Ok(format!("{}<router-view />\n", ind))
            }

            AuraNode::Link { to, text, href, children, .. } => {
                // Handle different link types:
                // 1. External link with href: <a href="...">
                // 2. Router link with to: <router-link to="...">
                if !href.is_empty() {
                    // External link
                    let text_content = if text.is_empty() {
                        let mut children_html = String::new();
                        for child in children {
                            children_html.push_str(&self.node_to_html(child, indent + 1)?);
                        }
                        children_html
                    } else {
                        text.clone()
                    };
                    Ok(format!("{}<a href=\"{}\">{}</a>\n", ind, href, text_content.trim()))
                } else {
                    // Vue Router link
                    self.needs_router = true;
                    let children_html = if text.is_empty() {
                        let mut html = String::new();
                        for child in children {
                            html.push_str(&self.node_to_html(child, indent + 1)?);
                        }
                        html
                    } else {
                        text.clone()
                    };
                    Ok(format!("{}<router-link to=\"{}\" class=\"group block\" active-class=\"\" exact-active-class=\"router-link-exact-active\">\n{}{}</router-link>\n", ind, to, children_html, ind))
                }
            }
        }
    }

    /// Generate HTML for interactive previewcard element
    fn generate_previewcard_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        _events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "  ".repeat(indent);

        // Extract props
        let id = if let Some(value) = props.get("id") {
            self.extract_string_value(value)
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    self.previewcard_counter += 1;
                    format!("preview{}", self.previewcard_counter)
                })
        } else {
            self.previewcard_counter += 1;
            format!("preview{}", self.previewcard_counter)
        };

        // Capitalize first letter for variable names
        // Convert kebab-case to PascalCase (e.g., "card-basic" -> "CardBasic")
        let id_cap = id.split('-')
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<String>();

        // Also create a lowercase version for code variable names (still camelCase)
        let id_lower = id.split('-')
            .enumerate()
            .map(|(i, part)| {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => {
                        if i == 0 {
                            c.to_lowercase().collect::<String>() + chars.as_str()
                        } else {
                            c.to_uppercase().collect::<String>() + chars.as_str()
                        }
                    }
                }
            })
            .collect::<String>();

        // Generate Auto code from children if not provided
        let auto_code = if let Some(value) = props.get("auto") {
            self.extract_string_value(value).unwrap_or_default().to_string()
        } else {
            // Auto-generate Auto code from children
            let mut auto_code_parts = Vec::new();
            for child in children {
                auto_code_parts.push(self.node_to_auto_code(child, 0));
            }
            let generated = auto_code_parts.join("\n");
            if generated.is_empty() {
                "// Auto code not provided".to_string()
            } else {
                generated
            }
        };

        // Generate Vue code from children if not provided
        let vue_code = if let Some(value) = props.get("vue") {
            self.extract_string_value(value).unwrap_or_default().to_string()
        } else {
            // Auto-generate Vue code from children
            let mut vue_code_parts = Vec::new();
            for child in children {
                match self.node_to_html(child, 0) {
                    Ok(html) => vue_code_parts.push(html),
                    Err(_) => vue_code_parts.push("<!-- Error generating code -->".to_string()),
                }
            }
            let generated = vue_code_parts.join("\n");
            if generated.is_empty() {
                "// Vue code not provided".to_string()
            } else {
                generated
            }
        };

        // Store previewcard data for script generation
        self.previewcard_data.push(PreviewCardData {
            id: id_cap.clone(),
            auto_code: auto_code.clone(),
            vue_code: vue_code.clone(),
        });
        self.needs_copy_code = true;

        // Generate children HTML for preview area
        let mut children_html = String::new();
        for child in children {
            children_html.push_str(&self.node_to_html(child, indent + 3)?);
        }

        // Generate the full previewcard HTML
        let html = format!(
            r#"{ind}<!-- Merged {id_cap} Component -->
{ind}<div class="rounded-lg border overflow-hidden">
{ind}  <!-- Preview Area -->
{ind}  <div class="flex items-center justify-center p-4 min-h-[100px] bg-zinc-100 dark:bg-zinc-900">
{ind}    {children_html}{ind}  </div>
{ind}  <!-- Toggle Code Footer -->
{ind}  <div class="border-t">
{ind}    <button
{ind}      @click="show{id_cap}Code = !show{id_cap}Code"
{ind}      class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
{ind}    >
{ind}      <span class="font-medium">Code</span>
{ind}      <svg
{ind}        :class="show{id_cap}Code ? 'rotate-180' : ''"
{ind}        xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
{ind}        class="transition-transform duration-200"
{ind}      >
{ind}        <path d="m6 9 6 6 6-6"/>
{ind}      </svg>
{ind}    </button>
{ind}    <!-- Expandable Code Block -->
{ind}    <div v-if="show{id_cap}Code" class="border-t">
{ind}      <!-- Tabs (gray title bar) -->
{ind}      <div class="flex items-center justify-between bg-zinc-100 dark:bg-zinc-800">
{ind}        <div class="flex">
{ind}          <button
{ind}            @click="active{id_cap}Tab = 'auto'"
{ind}            :class="active{id_cap}Tab === 'auto' ? 'bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px' : 'text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent'"
{ind}            class="px-4 py-2 text-xs font-medium transition-colors"
{ind}          >
{ind}            Auto
{ind}          </button>
{ind}          <button
{ind}            @click="active{id_cap}Tab = 'vue'"
{ind}            :class="active{id_cap}Tab === 'vue' ? 'bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px' : 'text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent'"
{ind}            class="px-4 py-2 text-xs font-medium transition-colors"
{ind}          >
{ind}            Vue
{ind}          </button>
{ind}        </div>
{ind}        <button
{ind}          @click="copyCode(active{id_cap}Tab === 'auto' ? {id_lower}AutoCode : {id_lower}VueCode, '{id}')"
{ind}          class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"
{ind}        >
{ind}          <svg v-if="copiedCode !== '{id}'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
{ind}          <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
{ind}          {{{{ copiedCode === '{id}' ? 'Copied!' : 'Copy' }}}}
{ind}        </button>
{ind}      </div>
{ind}      <!-- Code content with syntax highlighting -->
{ind}      <pre class="overflow-x-auto p-4 text-sm bg-zinc-950 text-zinc-50"><code :class="'block font-mono !p-0 language-' + (active{id_cap}Tab === 'auto' ? 'auto' : 'html')">{{{{ active{id_cap}Tab === 'auto' ? {id_lower}AutoCode : {id_lower}VueCode }}}}</code></pre>
{ind}    </div>
{ind}  </div>
{ind}</div>
"#,
            ind = ind,
            id = id,
            id_cap = id_cap,
            id_lower = id_lower,
            children_html = children_html
        );

        Ok(html)
    }

    /// Generate HTML for codeblock element with copy button
    fn generate_codeblock_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        _events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "  ".repeat(indent);

        // Extract id prop or generate one
        let id = if let Some(value) = props.get("id") {
            self.extract_string_value(value)
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    self.codeblock_counter += 1;
                    format!("codeblock{}", self.codeblock_counter)
                })
        } else {
            self.codeblock_counter += 1;
            format!("codeblock{}", self.codeblock_counter)
        };

        // Extract lang prop (default: "text")
        let lang = if let Some(value) = props.get("lang") {
            self.extract_string_value(value).unwrap_or("text").to_string()
        } else {
            "text".to_string()
        };

        // Extract code content from props or children
        let code = if let Some(value) = props.get("code") {
            self.prop_to_text_content(value).unwrap_or_default()
        } else if let Some(value) = props.get("text") {
            self.prop_to_text_content(value).unwrap_or_default()
        } else {
            // Get text from children
            let mut code_parts = Vec::new();
            for child in children {
                if let AuraNode::Text(content) = child {
                    match content {
                        AuraTextContent::Literal(s) => code_parts.push(s.clone()),
                        AuraTextContent::Interpolated { template, .. } => code_parts.push(template.clone()),
                    }
                }
            }
            code_parts.join("\n")
        };

        // Convert kebab-case to camelCase for variable names (e.g., "install-button" -> "installButton")
        let id_camel = id.split('-')
            .enumerate()
            .map(|(i, part)| {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => {
                        if i == 0 {
                            c.to_lowercase().collect::<String>() + chars.as_str()
                        } else {
                            c.to_uppercase().collect::<String>() + chars.as_str()
                        }
                    }
                }
            })
            .collect::<String>();

        // Store codeblock data for script generation
        self.codeblock_data.push(CodeBlockData {
            id: id.clone(),
            code: code.clone(),
            lang: lang.clone(),
        });
        self.needs_copy_code = true;

        // Generate the codeblock HTML with copy button (gray title bar, dark code content)
        let html = format!(
            r#"{ind}<div class="relative rounded-lg border overflow-hidden">
{ind}  <div class="flex items-center justify-between px-4 py-3 bg-zinc-100 dark:bg-zinc-800 border-b">
{ind}    <span class="text-xs text-zinc-600 dark:text-zinc-400 font-medium">{lang}</span>
{ind}    <button
{ind}      @click="copyCode({id_camel}Code, '{id}')"
{ind}      class="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"
{ind}    >
{ind}      <svg v-if="copiedCode !== '{id}'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
{ind}      <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
{ind}      {{{{ copiedCode === '{id}' ? 'Copied!' : 'Copy' }}}}
{ind}    </button>
{ind}  </div>
{ind}  <pre class="p-4 text-sm bg-zinc-950 text-zinc-50 overflow-x-auto"><code class="block font-mono !p-0 language-{lang}">{{{{ {id_camel}Code }}}}</code></pre>
{ind}</div>
"#,
            ind = ind,
            id = id,
            id_camel = id_camel,
            lang = lang
        );

        Ok(html)
    }

    /// Convert AuraNode back to Auto source code string
    /// This is used to generate the Auto code for previewcard components
    fn node_to_auto_code(&self, node: &AuraNode, indent: usize) -> String {
        let ind = "    ".repeat(indent);

        match node {
            AuraNode::Element { tag, props, events, children, .. } => {
                let mut result = String::new();

                // Build props string
                let mut props_parts = Vec::new();
                for (key, value) in props {
                    let value_str = match value {
                        AuraPropValue::Expr(expr) => self.expr_to_auto_string(expr),
                        AuraPropValue::StyleBinding(bindings) => {
                            let binding_strs: Vec<String> = bindings.iter()
                                .map(|b| format!("{}: {}", b.style_name, self.expr_to_auto_string(&b.condition)))
                                .collect();
                            format!("{{{}}}", binding_strs.join(", "))
                        }
                    };
                    props_parts.push(format!("{}: {}", key, value_str));
                }

                // Build events string
                for (event_name, event) in events {
                    let _params_str = if event.params.is_empty() {
                        String::new()
                    } else {
                        format!("({})", event.params.join(", "))
                    };
                    props_parts.push(format!("{}: .{}", event_name, event.handler));
                }

                let props_str = if props_parts.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", props_parts.join(", "))
                };

                // Handle self-closing vs with children
                if children.is_empty() {
                    result.push_str(&format!("{}{}{} {{}}\n", ind, tag, props_str));
                } else {
                    result.push_str(&format!("{}{}{} {{\n", ind, tag, props_str));
                    for child in children {
                        result.push_str(&self.node_to_auto_code(child, indent + 1));
                    }
                    result.push_str(&format!("{}}}\n", ind));
                }

                result
            }

            AuraNode::Text(text_content) => {
                match text_content {
                    AuraTextContent::Literal(s) => {
                        format!("{}\"{}\"\n", ind, s)
                    }
                    AuraTextContent::Interpolated { template, bindings: _ } => {
                        // Show the template with bindings
                        format!("{}\"{}\"\n", ind, template)
                    }
                }
            }

            AuraNode::Conditional { condition, then_body, else_body, .. } => {
                let mut result = String::new();
                result.push_str(&format!("{}if {} {{\n", ind, condition));
                for child in then_body {
                    result.push_str(&self.node_to_auto_code(child, indent + 1));
                }
                result.push_str(&format!("{}}}\n", ind));
                if let Some(else_nodes) = else_body {
                    result.push_str(&format!("{}else {{\n", ind));
                    for child in else_nodes {
                        result.push_str(&self.node_to_auto_code(child, indent + 1));
                    }
                    result.push_str(&format!("{}}}\n", ind));
                }
                result
            }

            AuraNode::ForLoop { var, index, iterable, body, .. } => {
                let mut result = String::new();
                let loop_header = if let Some(idx) = index {
                    format!("for ({}, {}) in {}", var, idx, iterable)
                } else {
                    format!("for {} in {}", var, iterable)
                };
                result.push_str(&format!("{}{} {{\n", ind, loop_header));
                for child in body {
                    result.push_str(&self.node_to_auto_code(child, indent + 1));
                }
                result.push_str(&format!("{}}}\n", ind));
                result
            }

            AuraNode::Component { name, props, events, .. } => {
                let mut result = String::new();

                let mut props_parts = Vec::new();
                for (key, value) in props {
                    props_parts.push(format!("{}: {}", key, self.expr_to_auto_string(value)));
                }

                for (event_name, event) in events {
                    props_parts.push(format!("{}: .{}", event_name, event.handler));
                }

                let props_str = if props_parts.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", props_parts.join(", "))
                };

                result.push_str(&format!("{}{}{} {{}}\n", ind, name, props_str));
                result
            }

            // Plan 105: Router outlet and link
            AuraNode::Outlet => {
                format!("{}outlet\n", ind)
            }

            AuraNode::Link { to, text, href, children, .. } => {
                let mut result = String::new();
                // Generate appropriate link syntax based on which props are provided
                if !href.is_empty() {
                    // External link with href
                    if text.is_empty() {
                        result.push_str(&format!("{}link (href: \"{}\") {{\n", ind, href));
                        for child in children {
                            result.push_str(&self.node_to_auto_code(child, indent + 1));
                        }
                        result.push_str(&format!("{}}}\n", ind));
                    } else {
                        result.push_str(&format!("{}link (text: \"{}\", href: \"{}\") {{}}\n", ind, text, href));
                    }
                } else if !text.is_empty() && children.is_empty() {
                    // Shorthand form with just text and to
                    result.push_str(&format!("{}link (to: \"{}\", text: \"{}\") {{}}\n", ind, to, text));
                } else {
                    // Standard form with children
                    result.push_str(&format!("{}link (to: \"{}\") {{\n", ind, to));
                    for child in children {
                        result.push_str(&self.node_to_auto_code(child, indent + 1));
                    }
                    result.push_str(&format!("{}}}\n", ind));
                }
                result
            }
        }
    }

    /// Convert AuraExpr to Auto source code string
    fn expr_to_auto_string(&self, expr: &crate::ast::Expr) -> String {
        use crate::ast::Expr;
        use auto_val::Op;
        match expr {
            Expr::Int(n) => n.to_string(),
            Expr::I64(n) => n.to_string(),
            Expr::Float(n, _) | Expr::Double(n, _) => n.to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Str(s) | Expr::CStr(s) => format!("\"{}\"", s),
            Expr::Ident(name) => format!(".{}", name),
            Expr::Bina(left, op, right) => {
                let op_str = match op {
                    Op::Add => "+",
                    Op::Sub => "-",
                    Op::Mul => "*",
                    Op::Div => "/",
                    Op::Mod => "%",
                    Op::Eq => "==",
                    Op::Neq => "!=",
                    Op::Lt => "<",
                    Op::Le => "<=",
                    Op::Gt => ">",
                    Op::Ge => ">=",
                    Op::And => "&&",
                    Op::Or => "||",
                    _ => "+",
                };
                format!("{} {} {}", self.expr_to_auto_string(left), op_str, self.expr_to_auto_string(right))
            }
            Expr::Unary(op, operand) => {
                let op_str = match op {
                    Op::Sub => "-",
                    _ => "!",
                };
                format!("{}{}", op_str, self.expr_to_auto_string(operand))
            }
            Expr::Call(call) => {
                if let Expr::Dot(object, method) = call.name.as_ref() {
                    let args_str: Vec<String> = call.args.args.iter()
                        .filter_map(|a| match a {
                            crate::ast::Arg::Pos(e) | crate::ast::Arg::Pair(_, e) => Some(e.clone()),
                            _ => None,
                        })
                        .map(|a| self.expr_to_auto_string(&a))
                        .collect();
                    format!("{}.{}({})", self.expr_to_auto_string(object), method, args_str.join(", "))
                } else {
                    let args_str: Vec<String> = call.args.args.iter()
                        .filter_map(|a| match a {
                            crate::ast::Arg::Pos(e) | crate::ast::Arg::Pair(_, e) => Some(e.clone()),
                            _ => None,
                        })
                        .map(|a| self.expr_to_auto_string(&a))
                        .collect();
                    format!("{}({})", self.expr_to_auto_string(&call.name), args_str.join(", "))
                }
            }
            Expr::Array(elements) => {
                let elements_str: Vec<String> = elements.iter().map(|e| self.expr_to_auto_string(e)).collect();
                format!("[{}]", elements_str.join(", "))
            }
            Expr::Object(pairs) => {
                let pairs: Vec<String> = pairs.iter()
                    .map(|p| format!("{}: {}", p.key.to_astr(), self.expr_to_auto_string(&p.value)))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            Expr::Closure(closure) => {
                let params_str: Vec<String> = closure.params.iter().map(|p| p.name.to_string()).collect();
                format!("|{}| {}", params_str.join(", "), self.expr_to_auto_string(&closure.body))
            }
            Expr::Dot(object, field) => {
                format!("{}.{}", self.expr_to_auto_string(object), field)
            }
            Expr::NavCall { path, params } => {
                let params_str: Vec<String> = params.iter()
                    .map(|p| format!("{}: {}", p.key.to_astr(), self.expr_to_auto_string(&p.value)))
                    .collect();
                format!("Nav.to(\"{}\", {{ {} }})", self.expr_to_auto_string(path), params_str.join(", "))
            }
            Expr::Index(target, index) => {
                format!("{}[{}]", self.expr_to_auto_string(target), self.expr_to_auto_string(index))
            }
            _ => "/* unsupported expr */".to_string(),
        }
    }

    /// Convert AURA condition to Vue expression
    fn convert_condition(&mut self, condition: &str) -> String {
        // Convert .var to var, .len to .length, etc.
        let mut result = condition.trim().to_string();

        // Replace .len() with .length (JavaScript property, not method)
        result = result.replace(".len()", ".length");
        result = result.replace(".len", ".length");
        // Handle spaced-out .len ( ) from parse_condition_expr
        result = result.replace(" .len ( )", ".length");
        result = result.replace(" len ( )", ".length");
        // Plan 345 (gap N1): .contains → .includes (JS uses .includes, not .contains)
        result = result.replace(".contains(", ".includes(");
        // Plan 043 M5: Auto's None/nil literal → JS null in view conditions.
        // parse_condition_expr emits tokens space-separated, so `None`/`nil`
        // appear as standalone words. Replace them with `null`.
        let mut out = String::with_capacity(result.len());
        for tok in result.split(' ') {
            if !out.is_empty() {
                out.push(' ');
            }
            match tok {
                "None" | "nil" => out.push_str("null"),
                _ => out.push_str(tok),
            }
        }
        result = out;

        // Remove leading dot from state references (.count -> count)
        // Pattern: .identifier (at word boundary)
        let mut converted = String::new();
        let chars: Vec<char> = result.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '.' && (i == 0 || !chars[i-1].is_alphanumeric()) {
                // Check if this is a number (like 0.5)
                if i + 1 < chars.len() && chars[i+1].is_ascii_digit() {
                    converted.push('.');
                    i += 1;
                    continue;
                }
                // Skip the dot (remove state prefix)
                i += 1;
                continue;
            }
            converted.push(chars[i]);
            i += 1;
        }

        // Replace double quotes with single quotes for Vue template compatibility
        // (v-if="currentPage == 'button'" not v-if="currentPage == "button"")
        let mut final_result = String::new();
        let mut in_string = false;
        for c in converted.chars() {
            if c == '"' {
                if in_string {
                    final_result.push('\'');  // End of string, use single quote
                    in_string = false;
                } else {
                    final_result.push('\'');  // Start of string, use single quote
                    in_string = true;
                }
            } else {
                final_result.push(c);
            }
        }

        // Handle leftover "length ( )" from parse_condition_expr + dot removal
        final_result = final_result.replace("length ( )", "length");
        final_result = final_result.replace("length ()", "length");

        final_result
    }

    /// Map AutoUI tag to HTML tag or shadcn-vue component
    fn map_tag(&mut self, tag: &str, self_closing: bool) -> String {
        // Priority: known sub-widgets > external (widget `use`) components >
        // shadcn components > HTML fallback
        // If tag matches a known sub-widget, treat as custom component reference
        if self.known_sub_widgets.contains(tag) {
            if !self.component_refs.contains(&tag.to_string()) {
                self.component_refs.push(tag.to_string());
            }
            return tag.to_string();
        }

        // Widget-declared external components (`use { component: ... }`) win
        // over the built-in registry so user declarations can shadow/extend it.
        if let Some(comp) = self.ext_components.get(tag) {
            return comp.name.clone();
        }

        // If in shadcn mode and tag has a shadcn component, use it
        if self.is_shadcn() {
            // nav-link maps to router-link (not a shadcn component)
            if tag == "nav-link" {
                return "router-link".to_string();
            }
            // theme-toggle maps to ThemeToggle custom component
            if tag == "theme-toggle" || tag == "theme_toggle" {
                self.component_refs.push("ThemeToggle".to_string());
                self.use_theme_toggle = true;
                return "ThemeToggle".to_string();
            }
            // Toast sub-components map to plain HTML (vue-sonner uses Toaster only)
            if tag == "toast" {
                return "div".to_string();
            }
            if tag == "toast-title" {
                return "span".to_string();
            }
            if tag == "toast-description" {
                return "span".to_string();
            }
            if let Some(component_name) = self.shadcn_component_name(tag) {
                self.register_shadcn_component(tag);
                return component_name.to_string();
            }
        }

        // Fallback to plain HTML tags
        match tag {
            // Layout (no shadcn components, use Tailwind)
            "col" | "column" | "Col" | "Column" => "div".to_string(),
            "row" | "Row" => "div".to_string(),
            "grid" | "Grid" => "div".to_string(),
            "scroll" | "Scroll" => "div".to_string(),
            "container" | "Container" => "div".to_string(),
            "center" | "Center" => "div".to_string(),

            // HTML5 semantic elements
            "header" | "Header" => "header".to_string(),
            "nav" | "Nav" => "nav".to_string(),
            "main" | "Main" => "main".to_string(),
            "section" | "Section" => "section".to_string(),
            "aside" | "Aside" => "aside".to_string(),
            "footer" | "Footer" => "footer".to_string(),
            "article" | "Article" => "article".to_string(),

            // Content
            "button" | "Button" => "button".to_string(),
            "input" | "Input" => "input".to_string(),
            "textarea" | "Textarea" => "textarea".to_string(),
            "checkbox" | "Checkbox" => "input".to_string(),
            "toggle" | "Toggle" => "button".to_string(),
            "select" | "Select" => "select".to_string(),
            "option" | "Option" => "option".to_string(),
            "link" | "Link" => "a".to_string(),
            "codeblock" | "code-block" | "CodeBlock" | "Codeblock" => "pre".to_string(),
            "codepane" | "code-pane" | "CodePane" => "div".to_string(),
            "previewcard" | "preview-card" | "PreviewCard" => "div".to_string(),

            // Typography (no shadcn components) - PascalCase maps to lowercase HTML
            "h1" | "H1" => "h1".to_string(),
            "h2" | "H2" => "h2".to_string(),
            "h3" | "H3" => "h3".to_string(),
            "h4" | "H4" => "h4".to_string(),
            "h5" | "H5" => "h5".to_string(),
            "h6" | "H6" => "h6".to_string(),
            "text" | "Text" => "span".to_string(),
            "label" | "Label" => "label".to_string(),
            "span" | "Span" => "span".to_string(),
            "p" | "P" => "p".to_string(),

            // Data
            "table" | "Table" => "table".to_string(),
            "thead" | "Thead" => "thead".to_string(),
            "tbody" | "Tbody" => "tbody".to_string(),
            "tr" | "Tr" => "tr".to_string(),
            "th" | "Th" => "th".to_string(),
            "td" | "Td" => "td".to_string(),
            "tree" | "Tree" => "ul".to_string(),
            "tree_item" | "tree-item" | "TreeItem" => "li".to_string(),

            // Navigation
            "tabs" | "Tabs" => "div".to_string(),
            "tab" | "Tab" => "button".to_string(),

            // Overlay
            "modal" | "Modal" => "div".to_string(),
            "tooltip" | "Tooltip" => "span".to_string(),

            // Form
            "slider" | "Slider" => "input".to_string(),
            "radio" | "Radio" => "input".to_string(),
            "radiogroup" | "radio-group" | "RadioGroup" => "div".to_string(),

            // Feedback
            "progress" | "Progress" => "progress".to_string(),
            "badge" | "Badge" => "span".to_string(),
            "spinner" | "Spinner" => "div".to_string(),

            // Display - Card is a special component, not a plain div
            "card" => "div".to_string(),
            "avatar" | "Avatar" => "img".to_string(),
            "aspectratio" | "aspect-ratio" | "AspectRatio" => "div".to_string(),

            // Media
            "image" | "Image" => "img".to_string(),
            "img" | "Img" => "img".to_string(),
            "icon" | "Icon" => "span".to_string(),

            // Utility
            "divider" | "Divider" => "hr".to_string(),
            "spacer" | "Spacer" => "div".to_string(),

            // Special
            "div" => "div".to_string(),
            "+" => if self_closing { "span".to_string() } else { "span".to_string() },
            "-" => if self_closing { "span".to_string() } else { "span".to_string() },

            _ => {
                // Check if it's a PascalCase component name
                if tag.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    self.component_refs.push(tag.to_string());
                    tag.to_string()
                } else {
                    "div".to_string()
                }
            }
        }
    }

    /// Normalize tag name to lowercase for matching
    fn normalize_tag(tag: &str) -> &str {
        // Handle PascalCase to lowercase conversion for common patterns
        match tag {
            "Col" | "Column" => "col",
            "Row" => "row",
            "Grid" => "grid",
            "Scroll" => "scroll",
            "Container" => "container",
            "Center" => "center",
            "Header" => "header",
            "Nav" => "nav",
            "Main" => "main",
            "Section" => "section",
            "Aside" => "aside",
            "Footer" => "footer",
            "Article" => "article",
            "Button" => "button",
            "Input" => "input",
            "Textarea" => "textarea",
            "Checkbox" => "checkbox",
            "Toggle" => "toggle",
            "Select" => "select",
            "Link" => "link",
            "H1" => "h1",
            "H2" => "h2",
            "H3" => "h3",
            "H4" => "h4",
            "H5" => "h5",
            "H6" => "h6",
            "Text" => "text",
            "Label" => "label",
            "P" => "p",
            "Table" => "table",
            "Thead" => "thead",
            "Tbody" => "tbody",
            "Tr" => "tr",
            "Th" => "th",
            "Td" => "td",
            "Tree" => "tree",
            "TreeItem" => "tree_item",
            "Tabs" => "tabs",
            "Tab" => "tab",
            "Modal" => "modal",
            "Tooltip" => "tooltip",
            "Slider" => "slider",
            "RadioGroup" => "radiogroup",
            "Progress" => "progress",
            "Badge" => "badge",
            "Spinner" => "spinner",
            "Card" => "card",
            "CardHeader" => "cardheader",
            "CardTitle" => "cardtitle",
            "CardDescription" => "carddescription",
            "CardContent" => "cardcontent",
            "CardFooter" => "cardfooter",
            "Avatar" => "avatar",
            "AspectRatio" => "aspectratio",
            "Image" => "image",
            "Img" => "img",
            "Icon" => "icon",
            "Divider" => "divider",
            "Separator" => "separator",
            "Spacer" => "spacer",
            _ => tag,
        }
    }

    /// Render a key for a JS object literal: bare when it is a valid JS
    /// identifier, single-quoted otherwise (CSS props like `z-index`, class
    /// names like `line-through`).
    fn js_obj_key(name: &str) -> String {
        let valid = !name.is_empty()
            && name.chars().next().map(|c| c.is_ascii_alphabetic() || c == '_' || c == '$').unwrap_or(false)
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
        if valid {
            name.to_string()
        } else {
            format!("'{}'", name.replace('\\', "\\\\").replace('\'', "\\'"))
        }
    }

    /// Render a `style_obj: { ... }` map as a Vue `:style` object binding.
    /// Values are arbitrary expressions (state refs, f-string px concat, …).
    /// The `as any` cast keeps loosely-typed values (e.g. a plain `string`
    /// state var for `visibility`, whose CSS type is a literal union)
    /// assignable to Vue's `StyleValue`.
    fn style_obj_to_vue(&self, bindings: &[AuraStyleBinding]) -> String {
        let parts: Vec<String> = bindings.iter()
            .map(|b| {
                let v = self.expr_to_vue_bound_value(&b.condition).unwrap_or_else(|_| "null".to_string());
                format!("{}: {}", Self::js_obj_key(&b.style_name), v)
            })
            .collect();
        format!("({{ {} }} as any)", parts.join(", "))
    }

    /// Extract Tailwind classes from tag and props
    /// Returns (static_classes, dynamic_class_binding)
    fn extract_classes(&self, tag: &str, props: &HashMap<String, AuraPropValue>) -> (String, Option<String>) {
        let mut classes = Vec::new();
        let mut dynamic_binding: Option<String> = None;

        // Normalize tag to lowercase for matching
        let normalized_tag = Self::normalize_tag(tag);

        // In shadcn mode, skip default classes for components that have shadcn versions
        // (shadcn components have their own styling).
        // However, layout primitives (row, col, etc.) always need their flex classes
        // regardless of mode — they map to <div> and have no shadcn styling of their own.
        let layout_primitives = ["row", "col", "column", "grid", "scroll", "center", "container"];
        let is_layout_primitive = layout_primitives.contains(&normalized_tag);
        let skip_defaults = !is_layout_primitive && self.is_shadcn() && self.widget_registry.is_backend_supported("vue", tag);

        // Check if user has provided a class or style attribute
        let has_user_class = props.contains_key("class") || props.contains_key("style");

        // For elements that should skip default classes when user provides their own class.
        // This covers semantic HTML elements, layout elements, typography, and form elements
        // that may need fully custom styling (e.g., TodoMVC uses todomvc-app-css).
        let user_class_skip_elements = [
            // Semantic HTML5
            "header", "nav", "main", "aside", "footer", "article", "section",
            // Typography
            "h1", "h2", "h3", "h4", "h5", "h6", "text", "p",
            // Form
            "button", "input", "checkbox", "link", "label",
            // Data
            "tree", "tree_item", "tree-item",
        ];
        let skip_semantic_defaults = has_user_class && user_class_skip_elements.contains(&normalized_tag);

        // Extract gap prop for layout elements
        let gap_class = if let Some(value) = props.get("gap") {
            match value {
                AuraPropValue::Expr(crate::ast::Expr::Str(s)) => format!("gap-{}", s),
                _ => "gap-4".to_string(),
            }
        } else {
            "gap-4".to_string()
        };

        // Default classes based on tag (only in Plain mode or for non-shadcn elements)
        if !skip_defaults && !skip_semantic_defaults {
            match normalized_tag {
                // Layout
                "col" | "column" => classes.push(format!("flex flex-col {}", gap_class)),
                "row" => classes.push(format!("flex flex-row {}", gap_class)),
                "grid" => classes.push("grid".to_string()),
                "scroll" => classes.push("overflow-auto".to_string()),
                "container" => classes.push("max-w-7xl mx-auto".to_string()),
                "center" => classes.push("flex flex-col items-center justify-center h-full".to_string()),

                // HTML5 semantic elements (only add defaults if user hasn't provided class)
                "header" => classes.push("w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60".to_string()),
                "nav" => classes.push("flex items-center gap-4".to_string()),
                "main" => classes.push("flex-1".to_string()),
                "aside" => classes.push("w-64 border-r bg-background".to_string()),
                "footer" => classes.push("w-full border-t bg-background".to_string()),
                "article" => classes.push("prose max-w-none".to_string()),

                // Typography
                "h1" => {
                    // Don't add default typography classes - let CSS handle sizing
                }
                "h2" => classes.push("text-2xl font-semibold tracking-tight mt-8".to_string()),
                "h3" => classes.push("text-xl font-semibold".to_string()),
                "text" => classes.push("text-muted-foreground leading-7".to_string()),

                // Content
                "button" => classes.push("px-4 py-2 rounded".to_string()),
                "input" => classes.push("border rounded px-2 py-1".to_string()),
                "textarea" => classes.push("border rounded px-2 py-1".to_string()),
                "checkbox" => classes.push("w-4 h-4".to_string()),
                "toggle" => classes.push("relative w-10 h-6 rounded-full".to_string()),
                "select" => classes.push("border rounded px-2 py-1".to_string()),
                "link" => classes.push("text-sm font-medium text-muted-foreground hover:text-foreground transition-colors cursor-pointer".to_string()),
                "codeblock" | "code-block" => classes.push("relative rounded-lg border bg-zinc-950 text-zinc-50 overflow-x-auto".to_string()),
                "codepane" | "code-pane" => classes.push("relative rounded-lg border bg-zinc-950 text-zinc-50 overflow-hidden".to_string()),
                "previewcard" | "preview-card" => classes.push("rounded-lg border overflow-hidden".to_string()),
                "label" => {
                    // Don't add default classes for native <label> elements
                    // (shadcn Label component is a separate widget, not plain label)
                }

                // Data
                "table" => classes.push("w-full border-collapse".to_string()),
                "thead" => classes.push("bg-muted/50".to_string()),
                "th" => classes.push("border px-4 py-2 text-left font-semibold".to_string()),
                "td" => classes.push("border px-4 py-2".to_string()),
                "tree" => classes.push("list-none pl-4".to_string()),
                "tree_item" | "tree-item" => classes.push("py-1".to_string()),

                // Navigation
                "tabs" => classes.push("flex border-b".to_string()),
                "tabslist" | "tabs-list" => classes.push("inline-flex h-9 items-center justify-center rounded-lg bg-muted p-1 text-muted-foreground".to_string()),
                "tabstrigger" | "tabs-trigger" => classes.push("inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50".to_string()),
                "tabscontent" | "tabs-content" => classes.push("mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2".to_string()),
                "tab" => classes.push("px-4 py-2 border-b-2 border-transparent".to_string()),

                // Overlay
                "modal" => classes.push("fixed inset-0 bg-black/80 flex items-center justify-center".to_string()),

                // Form
                "slider" => classes.push("w-full".to_string()),
                "radiogroup" | "radio-group" => classes.push("flex flex-col gap-2".to_string()),

                // Feedback
                "progress" => classes.push("w-full h-2 rounded".to_string()),
                "badge" => classes.push("px-2 py-1 text-xs rounded-full".to_string()),
                "spinner" => classes.push("animate-spin w-6 h-6 border-2 border-muted-foreground border-t-primary rounded-full".to_string()),

                // Display
                "card" => classes.push("rounded-lg border bg-card text-card-foreground shadow-sm".to_string()),
                "cardheader" | "card-header" => classes.push("flex flex-col space-y-1.5 p-6".to_string()),
                "cardtitle" | "card-title" => classes.push("text-lg font-semibold leading-none tracking-tight".to_string()),
                "carddescription" | "card-description" => classes.push("text-sm text-muted-foreground".to_string()),
                "cardcontent" | "card-content" => {
                    // Don't add default padding - let users control via class prop
                }
                "cardfooter" | "card-footer" => classes.push("flex items-center p-6 pt-0".to_string()),
                "avatar" => classes.push("w-10 h-10 rounded-full".to_string()),
                "aspectratio" | "aspect-ratio" => classes.push("relative w-full".to_string()),

                // Media
                "image" => classes.push("max-w-full".to_string()),
                "icon" => classes.push("w-5 h-5".to_string()),

                // Utility
                "divider" => classes.push("shrink-0 bg-border".to_string()),
                "separator" => classes.push("shrink-0 bg-border".to_string()),
                "spacer" => classes.push("flex-1".to_string()),

                _ => {}
            }
        }

        // Process 'style' (dynamic class binding) and 'class' (static classes) independently.
        // Both can coexist: class provides static Tailwind utilities, style provides dynamic :class.
        // Process 'style' prop first (generates dynamic :class binding)
        if let Some(value) = props.get("style") {
            match value {
                AuraPropValue::StyleBinding(bindings) => {
                    // Generate dynamic class binding: { completed: todo.done, editing: todo.editing }
                    // Use expr_to_vue_bound_value (no .value suffix) because Vue templates auto-unwrap refs
                    let binding_strs: Vec<String> = bindings.iter()
                        .map(|b| {
                            let cond = self.expr_to_vue_bound_value(&b.condition).unwrap_or_else(|_| "false".to_string());
                            // Class names may contain '-' (e.g. "line-through")
                            // — quote keys that aren't valid JS identifiers.
                            format!("{}: {}", Self::js_obj_key(&b.style_name), cond)
                        })
                        .collect();
                    dynamic_binding = Some(format!("{{ {} }}", binding_strs.join(", ")));
                }
                AuraPropValue::Expr(crate::ast::Expr::Str(s)) => {
                    // Dedup: for layout primitives, split user classes and skip any already present
                    if is_layout_primitive {
                        for c in s.split_whitespace() {
                            let existing: Vec<&str> = classes.iter().flat_map(|cl| cl.split_whitespace()).collect();
                            if !existing.contains(&c) {
                                classes.push(c.to_string());
                            }
                        }
                    } else {
                        classes.push(s.to_string());
                    }
                }
                AuraPropValue::Expr(crate::ast::Expr::If(if_stmt)) => {
                    // Plan 346: conditional style → Vue :class ternary.
                    if let Some(branch) = if_stmt.branches.first() {
                        let cond_str = self.expr_to_vue_bound_value(&branch.cond).unwrap_or_else(|_| "false".to_string());
                        // Extract string from body: check Stmt::Return(Expr::Str),
                        // Stmt::Expr(Expr::Str), or bare Expr::Str
                        let extract_str = |stmts: &[crate::ast::Stmt]| -> String {
                            for st in stmts {
                                match st {
                                    crate::ast::Stmt::Return(e) => {
                                        if let crate::ast::Expr::Str(s) = e.as_ref() { return s.to_string(); }
                                    }
                                    crate::ast::Stmt::Expr(e) => {
                                        if let crate::ast::Expr::Str(s) = e { return s.to_string(); }
                                    }
                                    _ => {}
                                }
                            }
                            String::new()
                        };
                        let then_str = extract_str(&branch.body.stmts);
                        let else_str = if_stmt.else_.as_ref()
                            .map(|body| extract_str(&body.stmts))
                            .unwrap_or_default();
                        dynamic_binding = Some(format!("{} ? '{}' : '{}'", cond_str, then_str, else_str));
                    }
                }
                _ => {}
            }
        }
        // Process 'class' prop (static Tailwind classes)
        if let Some(value) = props.get("class") {
            match value {
                AuraPropValue::Expr(crate::ast::Expr::Str(s)) => {
                    if is_layout_primitive {
                        for c in s.split_whitespace() {
                            let existing: Vec<&str> = classes.iter().flat_map(|cl| cl.split_whitespace()).collect();
                            if !existing.contains(&c) {
                                classes.push(c.to_string());
                            }
                        }
                    } else {
                        classes.push(s.to_string());
                    }
                }
                _ => {}
            }
        }

        (classes.join(" "), dynamic_binding)
    }

    /// Plan 100: Infer TypeScript type from AuraExpr
    fn expr_to_ts_type(&self, expr: &crate::ast::Expr) -> String {
        use crate::ast::Expr;
        match expr {
            Expr::Int(_) | Expr::I64(_) | Expr::Uint(_) | Expr::U64(_)
            | Expr::I8(_) | Expr::U8(_) | Expr::Byte(_)
            | Expr::Float(_, _) | Expr::Double(_, _) => "number".to_string(),
            Expr::Bool(_) => "boolean".to_string(),
            Expr::Str(_) | Expr::CStr(_) => "string".to_string(),
            Expr::Ident(name) => {
                let resolved = if name.starts_with('.') { &name[1..] } else { name.as_str() };
                // Prefer declared prop/state types over name heuristics.
                if let Some(ty) = self.prop_types.get(resolved).or_else(|| self.state_types.get(resolved)) {
                    return ty.clone();
                }
                // Try to infer type from state variable name
                if name.starts_with("is_") || name.starts_with("has_") {
                    "boolean".to_string()
                } else {
                    "number".to_string()  // Default to number for state refs
                }
            }
            // `.field` / `self.field` — resolve like the bare field name.
            Expr::Dot(object, field) => {
                if matches!(object.as_ref(), Expr::Ident(n) if n.as_str() == "self" || n.as_str() == ".") {
                    return self.expr_to_ts_type(&Expr::Ident(field.clone()));
                }
                "any".to_string()
            }
            Expr::Bina(left, op, right) => {
                use auto_val::Op;
                match op {
                    // Comparisons and logical ops always yield booleans.
                    Op::Eq | Op::Neq | Op::Lt | Op::Le | Op::Gt | Op::Ge
                    | Op::And | Op::Or => "boolean".to_string(),
                    // `+` is string concatenation when either side is a string
                    // (JS semantics) — don't blindly infer number.
                    Op::Add => {
                        let lt = self.expr_to_ts_type(left);
                        let rt = self.expr_to_ts_type(right);
                        if lt == "string" || rt == "string" {
                            "string".to_string()
                        } else if lt == "number" && rt == "number" {
                            "number".to_string()
                        } else {
                            "any".to_string()
                        }
                    }
                    Op::Sub | Op::Mul | Op::Div | Op::Mod => "number".to_string(),
                    _ => "any".to_string(),
                }
            }
            Expr::Unary(op, _) => {
                if matches!(op, auto_val::Op::Not) {
                    "boolean".to_string()
                } else {
                    "number".to_string()
                }
            }
            Expr::Array(_) => "any[]".to_string(),
            _ => "any".to_string(),  // Default fallback
        }
    }

    /// Check if LogicPayload contains NavCall (Plan 105)
    fn payload_has_nav_call(payload: &LogicPayload) -> bool {
        match payload {
            LogicPayload::AstStmts(_) => false, // NavCall handled at view tree level
            LogicPayload::Bytecode(_) => false, // Can't analyze bytecode
        }
    }

    /// Check if widget uses router features (Plan 105)
    fn widget_needs_router(&self, widget: &AuraWidget) -> bool {
        // Check handlers for NavCall
        for payload in widget.handlers.values() {
            if Self::payload_has_nav_call(payload) {
                return true;
            }
        }
        false
    }

    /// Check if LogicPayload contains route access (Plan 235)
    fn payload_has_route_access(payload: &LogicPayload) -> bool {
        match payload {
            LogicPayload::AstStmts(stmts) => {
                crate::ui_gen::ts_adapter::stmts_have_route_access(stmts)
            }
            LogicPayload::Bytecode(_) => false,
        }
    }

    /// Check if widget uses route features (Plan 235)
    fn widget_needs_route(widget: &AuraWidget) -> bool {
        for payload in widget.handlers.values() {
            if Self::payload_has_route_access(payload) {
                return true;
            }
        }
        false
    }

    /// Escape a string for use in JavaScript single-quoted string literals.
    fn escape_js_string(s: &str) -> String {
        s.replace("\\", "\\\\")
            .replace("'", "\\'")
            .replace("\n", "\\n")
            .replace("\r", "\\r")
            .replace("\t", "\\t")
    }

    /// Convert AuraExpr to JS value string
    fn expr_to_js(&self, expr: &crate::ast::Expr) -> GenResult<String> {
        use crate::ast::Expr;
        use auto_val::Op;
        match expr {
            Expr::Str(s) | Expr::CStr(s) => Ok(format!("'{}'", Self::escape_js_string(s.as_str()))),
            Expr::Int(n) => Ok(n.to_string()),
            Expr::I8(n) => Ok((*n as i32).to_string()),
            Expr::U8(n) => Ok((*n as i32).to_string()),
            Expr::Byte(n) => Ok((*n as i32).to_string()),
            Expr::Uint(n) => Ok((*n as i32).to_string()),
            Expr::Char(c) => Ok((*c as i32).to_string()),
            Expr::I64(n) => Ok(n.to_string()),
            Expr::U64(n) => Ok((*n as i64).to_string()),
            Expr::Float(n, _) | Expr::Double(n, _) => Ok(n.to_string()),
            Expr::Bool(b) => Ok(b.to_string()),
            Expr::Ident(name) => {
                let resolved = if name.starts_with('.') { &name[1..] } else { name.as_str() };
                if self.prop_names.contains(&resolved.to_string()) {
                    // Props: access via props.xxx (no .value, but need props. prefix in script)
                    Ok(format!("props.{}", resolved))
                } else if self.state_names.contains(&resolved.to_string()) {
                    Ok(format!("{}.value", resolved))
                } else {
                    Ok(resolved.to_string())
                }
            }
            Expr::Bina(left, op, right) => {
                let left_js = self.expr_to_js(left)?;
                let right_js = self.expr_to_js(right)?;
                let op_js = Self::op_to_js(op);
                Ok(format!("{} {} {}", left_js, op_js, right_js))
            }
            Expr::Unary(op, operand) => {
                let operand_js = self.expr_to_js(operand)?;
                let op_js = match op {
                    Op::Not => "!",
                    Op::Sub => "-",
                    _ => "!",
                };
                Ok(format!("{}{}", op_js, operand_js))
            }
            Expr::Dot(object, field) => {
                // Method call pattern: object.method(args) is parsed as
                // Call(Dot(object, method), args).
                // This branch handles plain field access.
                // `.field` is parsed as Dot(Ident("self"), field) — a widget
                // member reference. Resolve it against props/state instead of
                // emitting a nonexistent `self` object.
                if let Expr::Ident(name) = object.as_ref() {
                    if name.as_str() == "self" || name.as_str() == "." {
                        let field_name = field.as_str();
                        if self.prop_names.contains(&field_name.to_string()) {
                            return Ok(format!("props.{}", field_name));
                        } else if self.state_names.contains(&field_name.to_string()) {
                            return Ok(format!("{}.value", field_name));
                        } else {
                            // Computed props and locals are plain consts in
                            // <script setup>; computed refs need .value, but
                            // without widget context here the bare name is the
                            // least-wrong fallback (matches ts_adapter).
                            return Ok(field_name.to_string());
                        }
                    }
                }
                let object_js = self.expr_to_js(object)?;
                Ok(format!("{}.{}", object_js, field))
            }
            Expr::Call(call) => {
                // The call's name may be a Dot(object, method) — a method call.
                if let Expr::Dot(object, method) = call.name.as_ref() {
                    let method = method.clone();
                    let args: Vec<crate::ast::Expr> = call.args.args.iter()
                        .filter_map(|a| match a {
                            crate::ast::Arg::Pos(e) | crate::ast::Arg::Pair(_, e) => Some(e.clone()),
                            _ => None,
                        })
                        .collect();

                    // Plan 132: Check if this is an API function call
                    // Case 1: Direct API call like listusers() - object is API name
                    if let Expr::Ident(name) = object.as_ref() {
                        let resolved = if name.starts_with('.') { &name[1..] } else { name.as_str() };
                        if self.is_api_function(resolved) {
                            let args_js: Vec<String> = args.iter()
                                .map(|a| self.expr_to_js(a))
                                .collect::<Result<Vec<_>, _>>()?;
                            return Ok(format!("await {}({})", resolved, args_js.join(", ")));
                        }
                    }
                    // Case 2: self.<api_function>() - treat as direct API call
                    if let Expr::Ident(obj_name) = object.as_ref() {
                        if obj_name.as_str() == "self" && self.is_api_function(method.as_str()) {
                            let args_js: Vec<String> = args.iter()
                                .map(|a| self.expr_to_js(a))
                                .collect::<Result<Vec<_>, _>>()?;
                            return Ok(format!("await {}({})", method, args_js.join(", ")));
                        }
                    }
                    // Case 3: Any method call where method name is an API function
                    if self.is_api_function(method.as_str()) {
                        let args_js: Vec<String> = args.iter()
                            .map(|a| self.expr_to_js(a))
                            .collect::<Result<Vec<_>, _>>()?;
                        return Ok(format!("await {}({})", method, args_js.join(", ")));
                    }

                    let object_js = self.expr_to_js(object)?;
                    let args_js: Vec<String> = args.iter()
                        .map(|a| self.expr_to_js(a))
                        .collect::<Result<Vec<_>, _>>()?;
                    match method.as_str() {
                        "len" => Ok(format!("{}.length", object_js)),
                        // Plan 345 (gap N1): Auto `.contains` maps to JS `.includes`
                        "contains" => Ok(format!("{}.includes({})", object_js, args_js.join(", "))),
                        "to_string" => Ok(format!("{}.toString()", object_js)),
                        "to_int" => {
                            if args_js.is_empty() {
                                Ok(format!("parseInt({})", object_js))
                            } else {
                                Ok(format!("parseInt({}, {})", object_js, args_js.join(", ")))
                            }
                        }
                        "to_float" | "to_double" => Ok(format!("parseFloat({})", object_js)),
                        _ => Ok(format!("{}.{}({})", object_js, method, args_js.join(", "))),
                    }
                } else {
                    // Plain function call: func(args)
                    let name_js = self.expr_to_js(&call.name)?;
                    let args_js: Vec<String> = call.args.args.iter()
                        .filter_map(|a| match a {
                            crate::ast::Arg::Pos(e) | crate::ast::Arg::Pair(_, e) => Some(e.clone()),
                            _ => None,
                        })
                        .map(|a| self.expr_to_js(&a))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(format!("{}({})", name_js, args_js.join(", ")))
                }
            }
            Expr::Array(elems) => {
                let elems_js: Vec<String> = elems.iter()
                    .map(|e| self.expr_to_js(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("[{}]", elems_js.join(", ")))
            }
            Expr::Object(pairs) => {
                let pairs_js: Vec<String> = pairs.iter()
                    .map(|p| {
                        let v_js = self.expr_to_js(&p.value)?;
                        Ok(format!("{}: {}", p.key.to_astr(), v_js))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{{{}}}", pairs_js.join(", ")))
            }
            Expr::Closure(closure) => {
                let params: Vec<String> = closure.params.iter()
                    .map(|p| p.name.to_string())
                    .collect();
                let body_js = self.expr_to_js(&closure.body)?;
                Ok(format!("({}) => {}", params.join(", "), body_js))
            }
            Expr::Index(target, index) => {
                let target_js = self.expr_to_js(target)?;
                let index_js = self.expr_to_js(index)?;
                Ok(format!("{}[{}]", target_js, index_js))
            }
            Expr::NavCall { path, params } => {
                let path_js = self.expr_to_js(path)?;
                if params.is_empty() {
                    Ok(format!("router.push({})", path_js))
                } else {
                    let params_js: Vec<String> = params.iter()
                        .map(|p| {
                            self.expr_to_js(&p.value).map(|v_js| format!("{}: {}", p.key.to_astr(), v_js))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(format!("router.push({{ path: {}, query: {{ {} }} }})", path_js, params_js.join(", ")))
                }
            }
            // Plan 043 M5 #2: render a multi-statement computed body. The
            // computed wrapper at the call site is `computed(() => {expr_js})`,
            // so we emit `{ stmts; return tail; }` to produce a valid statement
            // arrow-body. Reuse the handler-body transpiler (same state/prop
            // rewriting rules) so the body sees `state.value` etc.
            Expr::Block(body) => {
                let mut ctx = crate::ui_gen::ts_adapter::AuraTsContext::new(self.state_names.iter().cloned().collect())
                    .with_props(self.prop_names.iter().cloned().collect())
                    .with_refs(self.template_refs.iter().cloned().collect());
                if !self.project_api_functions.is_empty() {
                    ctx = ctx.with_api_functions(self.project_api_functions.clone());
                }
                let body_js = crate::ui_gen::ts_adapter::transpile_handler_body(&body.stmts, &ctx);
                Ok(format!("{{ {} }}", body_js.trim()))
            }
            _ => Ok("undefined".to_string()),
        }
    }

    /// Convert binary operator (auto_val::Op) to JS
    fn op_to_js(op: &auto_val::Op) -> &'static str {
        match op {
            auto_val::Op::Add => "+",
            auto_val::Op::Sub => "-",
            auto_val::Op::Mul => "*",
            auto_val::Op::Div => "/",
            auto_val::Op::Mod => "%",
            auto_val::Op::Eq => "===",
            auto_val::Op::Neq => "!==",
            auto_val::Op::Lt => "<",
            auto_val::Op::Le => "<=",
            auto_val::Op::Gt => ">",
            auto_val::Op::Ge => ">=",
            auto_val::Op::And => "&&",
            auto_val::Op::Or => "||",
            _ => "+",
        }
    }

    // ========================================================================
    // Plan 132: API Call Detection for Async Handlers
    // ========================================================================

    /// List of known API function names (from @/lib/api)
    const API_FUNCTIONS: &'static [&'static str] = &[
        "listusers",
        "getuser",
        "getUser",
        "createUser",
        "updateUser",
        "deleteUser",
    ];

    /// Extract API function calls from a LogicPayload and track them
    fn extract_api_calls_from_payload(&mut self, payload: &LogicPayload) {
        match payload {
            LogicPayload::AstStmts(stmts) => {
                // Plan 132: Extract API calls from raw AST statements
                self.extract_api_calls_from_ast_stmts(stmts);
            }
            LogicPayload::Bytecode(_) => {
                // Bytecode not supported for API call detection
            }
        }
    }

    /// Extract API function calls from raw AST statements (Plan 132)
    fn extract_api_calls_from_ast_stmts(&mut self, stmts: &[crate::ast::Stmt]) {
        use crate::ast::{Expr, Stmt};

        fn walk_expr(expr: &Expr, api_fns: &[&str], used: &mut HashSet<String>) {
            match expr {
                Expr::Call(call) => {
                    let call_name = call.get_name_text_safe()
                        .map(|n| n.as_str().to_string())
                        .unwrap_or_default();
                    if !call_name.is_empty() {
                        if api_fns.contains(&call_name.as_str()) {
                            used.insert(call_name.clone());
                        }
                    }
                    // Recurse into args
                    for arg in &call.args.args {
                        walk_expr(&arg.get_expr(), api_fns, used);
                    }
                }
                Expr::Bina(l, _, r) => {
                    walk_expr(l, api_fns, used);
                    walk_expr(r, api_fns, used);
                }
                Expr::Unary(_, e) => walk_expr(e, api_fns, used),
                Expr::Dot(obj, _) => walk_expr(obj, api_fns, used),
                Expr::Array(items) => {
                    for item in items {
                        walk_expr(item, api_fns, used);
                    }
                }
                Expr::Block(body) => {
                    for stmt in &body.stmts {
                        walk_stmt(stmt, api_fns, used);
                    }
                }
                _ => {}
            }
        }

        fn walk_stmt(stmt: &Stmt, api_fns: &[&str], used: &mut HashSet<String>) {
            match stmt {
                Stmt::Expr(expr) => walk_expr(expr, api_fns, used),
                Stmt::Store(store) => walk_expr(&store.expr, api_fns, used),
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        walk_expr(&branch.cond, api_fns, used);
                        for stmt in &branch.body.stmts {
                            walk_stmt(stmt, api_fns, used);
                        }
                    }
                }
                Stmt::For(for_) => {
                    walk_expr(&for_.range, api_fns, used);
                    for stmt in &for_.body.stmts {
                        walk_stmt(stmt, api_fns, used);
                    }
                }
                Stmt::Block(body) => {
                    for stmt in &body.stmts {
                        walk_stmt(stmt, api_fns, used);
                    }
                }
                _ => {}
            }
        }

        let all_fns = self.all_api_functions();
        let all_fn_refs: Vec<&str> = all_fns.iter().map(|s| s.as_str()).collect();
        for stmt in stmts {
            walk_stmt(stmt, &all_fn_refs, &mut self.api_functions_used);
        }
    }

    /// Check if a handler payload contains API calls
    fn handler_has_api_calls(&self, payload: &LogicPayload) -> bool {
        match payload {
            LogicPayload::AstStmts(stmts) => {
                if self.project_api_functions.is_empty() {
                    crate::ui_gen::ts_adapter::stmts_contain_api_call(stmts)
                } else {
                    crate::ui_gen::ts_adapter::stmts_contain_api_call_with(stmts, &self.project_api_functions)
                }
            }
            LogicPayload::Bytecode(_) => false,
        }
    }

    /// Convert prop value to HTML attribute value
    /// For static values: produces `"value"`
    /// For dynamic values (StateRef, FieldAccess): produces `"name"` (caller must prefix with `:`)
    fn prop_to_attr_value(&self, value: &AuraPropValue) -> GenResult<String> {
        use crate::ast::Expr;
        match value {
            AuraPropValue::Expr(expr) => {
                match expr {
                    Expr::Ident(name) => {
                        let resolved = if name.starts_with('.') { &name[1..] } else { name.as_str() };
                        Ok(format!("\"{}\"", resolved))
                    }
                    Expr::Dot(object, field) => {
                        let obj_str = self.expr_to_vue_text(object)?;
                        Ok(format!("\"{}.{}\"", obj_str.trim_matches(|c| c == '{' || c == '}'), field))
                    }
                    _ => Ok(format!("\"{}\"", self.expr_to_vue_text(expr)?)),
                }
            }
            AuraPropValue::StyleBinding(_) => {
                // Class bindings are handled separately in extract_classes
                Ok("\"\"".to_string())
            }
        }
    }

    /// Convert prop value to text content (for rendering inside element)
    fn prop_to_text_content(&self, value: &AuraPropValue) -> GenResult<String> {
        match value {
            AuraPropValue::Expr(expr) => {
                self.expr_to_vue_text(expr)
            }
            AuraPropValue::StyleBinding(_) => {
                Ok("".to_string())
            }
        }
    }

    /// Convert AuraExpr to Vue template text (handles interpolation)
    /// Convert AuraExpr to raw Vue text (no {{ }} wrapping).
    /// Used internally by expr_to_vue_text for composing nested expressions.
    fn expr_to_vue_text_raw(&self, expr: &crate::ast::Expr) -> GenResult<String> {
        use crate::ast::Expr;
        match expr {
            Expr::Str(s) | Expr::CStr(s) => {
                let vue = self.convert_template_to_vue(s.as_str());
                let vue = vue.strip_prefix("{{ ")
                    .and_then(|v| v.strip_suffix(" }}"))
                    .map(|v| v.to_string())
                    .unwrap_or(vue);
                Ok(vue)
            }
            Expr::Int(n) => Ok(n.to_string()),
            Expr::I8(n) => Ok((*n as i32).to_string()),
            Expr::U8(n) => Ok((*n as i32).to_string()),
            Expr::Byte(n) => Ok((*n as i32).to_string()),
            Expr::Uint(n) => Ok((*n as i32).to_string()),
            Expr::Char(c) => Ok((*c as i32).to_string()),
            Expr::I64(n) => Ok(n.to_string()),
            Expr::U64(n) => Ok((*n as i64).to_string()),
            Expr::Float(f, _) | Expr::Double(f, _) => Ok(f.to_string()),
            Expr::Bool(b) => Ok(b.to_string()),
            Expr::Ident(name) => {
                let resolved = if name.starts_with('.') { &name[1..] } else { name.as_str() };
                Ok(resolved.to_string())
            }
            Expr::Dot(object, field) => {
                // Skip `self.` prefix — the view parser turns `.field` into
                // `self.field`, but Vue <script setup> has no `self`.
                if let Expr::Ident(name) = object.as_ref() {
                    if name == "self" {
                        return Ok(field.to_string());
                    }
                }
                let object_str = self.expr_to_vue_text_raw(object)?;
                Ok(format!("{}.{}", object_str, field))
            }
            Expr::Index(target, index) => {
                let target_str = self.expr_to_vue_text_raw(target)?;
                let index_str = self.expr_to_vue_text_raw(index)?;
                Ok(format!("{}[{}]", target_str, index_str))
            }
            Expr::FStr(fstr) => {
                // f-string as text content: literal parts stay raw,
                // interpolated expressions become {{ }} Vue mustaches.
                // (Previously fell through to the "value" placeholder.)
                let mut out = String::new();
                for part in &fstr.parts {
                    match part {
                        Expr::Str(s) | Expr::CStr(s) => out.push_str(s.as_str()),
                        other => {
                            let raw = self.expr_to_vue_text_raw(other)?;
                            out.push_str(&format!("{{{{ {} }}}}", raw));
                        }
                    }
                }
                Ok(out)
            }
            Expr::Bina(left, _, right) => {
                let left_str = self.expr_to_vue_text_raw(left)?;
                let right_str = self.expr_to_vue_text_raw(right)?;
                Ok(format!("{}{}", left_str, right_str))
            }
            Expr::Call(call) => {
                if let Expr::Dot(object, method) = call.name.as_ref() {
                    let method = method.clone();
                    let obj_str = self.expr_to_vue_text_raw(object)?;
                    let is_self = obj_str == "self";
                    let args: Vec<crate::ast::Expr> = call.args.args.iter()
                        .filter_map(|a| match a {
                            crate::ast::Arg::Pos(e) | crate::ast::Arg::Pair(_, e) => Some(e.clone()),
                            _ => None,
                        })
                        .collect();
                    match method.as_str() {
                        "to_string" => Ok(obj_str.clone()),
                        "len" => Ok(format!("{}.length", obj_str)),
                        "contains" => Ok(format!("{}.includes({})", obj_str, args.iter().map(|a| self.expr_to_vue_bound_value(a)).collect::<Result<Vec<_>, _>>()?.join(", "))),
                        _ => {
                            let args_str: Vec<String> = args.iter()
                                .map(|a| self.expr_to_vue_bound_value(a))
                                .collect::<Result<Vec<_>, _>>()?;
                            if is_self {
                                if args_str.is_empty() {
                                    Ok(format!("{}()", method))
                                } else {
                                    Ok(format!("{}({})", method, args_str.join(", ")))
                                }
                            } else {
                                if args_str.is_empty() {
                                    Ok(format!("{}.{}()", obj_str, method))
                                } else {
                                    Ok(format!("{}.{}({})", obj_str, method, args_str.join(", ")))
                                }
                            }
                        }
                    }
                } else {
                    let name_str = self.expr_to_vue_text_raw(&call.name)?;
                    let args_str: Vec<String> = call.args.args.iter()
                        .filter_map(|a| match a {
                            crate::ast::Arg::Pos(e) | crate::ast::Arg::Pair(_, e) => Some(e.clone()),
                            _ => None,
                        })
                        .map(|a| self.expr_to_vue_bound_value(&a))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(format!("{}({})", name_str, args_str.join(", ")))
                }
            }
            _ => Ok("value".to_string()),
        }
    }

    /// Convert AuraExpr to Vue template text with {{ }} wrapping for display.
    /// Uses expr_to_vue_text_raw internally and wraps the final result.
    fn expr_to_vue_text(&self, expr: &crate::ast::Expr) -> GenResult<String> {
        use crate::ast::Expr;
        // For compound expressions that produce their own {{ }},
        // use the raw version and wrap at the end.
        let raw = self.expr_to_vue_text_raw(expr)?;
        // If the raw result already contains {{ (e.g., from convert_template_to_vue),
        // or is a plain literal string / f-string, return as-is.
        // Otherwise wrap in {{ }}.
        if raw.starts_with("{{") || matches!(expr, Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_)) {
            Ok(raw)
        } else {
            Ok(format!("{{{{ {} }}}}", raw))
        }
    }

    /// Convert AuraExpr to Vue bound attribute value (for :prop="..." bindings).
    /// Used for chart props and other complex bindings where we need JavaScript
    /// expressions in Vue templates (state refs are kept bare, no .value).
    fn expr_to_vue_bound_value(&self, expr: &crate::ast::Expr) -> GenResult<String> {
        use crate::ast::Expr;
        use auto_val::Op;
        match expr {
            Expr::Str(s) | Expr::CStr(s) => Ok(format!("'{}'", Self::escape_js_string(s.as_str()))),
            Expr::Int(n) => Ok(n.to_string()),
            Expr::Float(n, _) | Expr::Double(n, _) => Ok(n.to_string()),
            Expr::Bool(b) => Ok(b.to_string()),
            Expr::Ident(name) => {
                let resolved = if name.starts_with('.') { &name[1..] } else { name.as_str() };
                Ok(resolved.to_string())
            }
            Expr::Dot(object, field) => {
                // State reference: .field or self.field → just the field name
                if let Expr::Ident(name) = object.as_ref() {
                    if name.as_str() == "." || name.as_str() == "self" {
                        return Ok(field.to_string());
                    }
                }
                let obj_str = self.expr_to_vue_bound_value(object)?;
                Ok(format!("{}.{}", obj_str, field))
            }
            Expr::Index(target, index) => {
                let target_str = self.expr_to_vue_bound_value(target)?;
                let index_str = self.expr_to_vue_bound_value(index)?;
                Ok(format!("{}[{}]", target_str, index_str))
            }
            Expr::Array(elems) => {
                let elems_vue: Vec<String> = elems.iter()
                    .map(|e| self.expr_to_vue_bound_value(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("[{}]", elems_vue.join(", ")))
            }
            Expr::Object(pairs) => {
                let pairs_vue: Vec<String> = pairs.iter()
                    .map(|p| {
                        let v_vue = self.expr_to_vue_bound_value(&p.value)?;
                        Ok(format!("{}: {}", p.key.to_astr(), v_vue))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{{{}}}", pairs_vue.join(", ")))
            }
            Expr::Bina(left, op, right) => {
                let left_str = self.expr_to_vue_bound_value(left)?;
                let right_str = self.expr_to_vue_bound_value(right)?;
                let op_str = match op {
                    Op::Eq => "==",
                    Op::Neq => "!=",
                    Op::Lt => "<",
                    Op::Gt => ">",
                    Op::Le => "<=",
                    Op::Ge => ">=",
                    Op::And => "&&",
                    Op::Or => "||",
                    Op::Add => "+",
                    Op::Sub => "-",
                    Op::Mul => "*",
                    Op::Div => "/",
                    Op::Mod => "%",
                    _ => "+",
                };
                Ok(format!("{} {} {}", left_str, op_str, right_str))
            }
            Expr::Unary(op, operand) => {
                let expr_str = self.expr_to_vue_bound_value(operand)?;
                match op {
                    Op::Not => Ok(format!("!{}", expr_str)),
                    Op::Sub => Ok(format!("-{}", expr_str)),
                    _ => Ok(format!("!{}", expr_str)),
                }
            }
            Expr::FStr(fstr) => {
                // f-string in a bound position → JS template literal:
                // f"${.top}px" → `${top}px` (needed e.g. for style_obj values).
                let mut out = String::from("`");
                for part in &fstr.parts {
                    match part {
                        Expr::Str(s) | Expr::CStr(s) => {
                            out.push_str(&s
                                .replace("\\", "\\\\")
                                .replace("`", "\\`")
                                .replace("${", "\\${"));
                        }
                        other => {
                            let v = self.expr_to_vue_bound_value(other)?;
                            out.push_str(&format!("${{{}}}", v));
                        }
                    }
                }
                out.push('`');
                Ok(out)
            }
            _ => Ok("null".to_string()),
        }
    }

    /// Emit a chart prop attribute.
    /// Literal strings become static attributes; everything else becomes a bound attribute.
    fn emit_chart_prop(&mut self, attrs: &mut Vec<String>, props: &HashMap<String, AuraPropValue>, key: &str, vue_attr: &str) {
        if let Some(value) = props.get(key) {
            match value {
                AuraPropValue::Expr(crate::ast::Expr::Str(s)) | AuraPropValue::Expr(crate::ast::Expr::CStr(s)) => {
                    attrs.push(format!("{}=\"{}\"", vue_attr, s));
                }
                AuraPropValue::Expr(expr) => {
                    if let Ok(v) = self.expr_to_vue_bound_value(expr) {
                        attrs.push(format!(":{}=\"{}\"", vue_attr, v));
                    }
                }
                _ => {}
            }
        }
    }

    /// Emit curve-type prop for charts, mapping string values to CurveType enum.
    fn emit_curve_type_prop(&mut self, attrs: &mut Vec<String>, props: &HashMap<String, AuraPropValue>) {
        if let Some(value) = props.get("curve-type").or_else(|| props.get("curve_type")) {
            self.use_curve_type = true;
            if let Some(s) = self.extract_string_value(value) {
                let mapped = match s {
                    "basis" => "CurveType.Basis",
                    "basisClosed" => "CurveType.BasisClosed",
                    "basisOpen" => "CurveType.BasisOpen",
                    "bundle" => "CurveType.Bundle",
                    "cardinal" => "CurveType.Cardinal",
                    "cardinalClosed" => "CurveType.CardinalClosed",
                    "cardinalOpen" => "CurveType.CardinalOpen",
                    "catmullRom" => "CurveType.CatmullRom",
                    "catmullRomClosed" => "CurveType.CatmullRomClosed",
                    "catmullRomOpen" => "CurveType.CatmullRomOpen",
                    "linear" => "CurveType.Linear",
                    "linearClosed" => "CurveType.LinearClosed",
                    "monotone" | "monotoneX" => "CurveType.MonotoneX",
                    "monotoneY" => "CurveType.MonotoneY",
                    "natural" => "CurveType.Natural",
                    "step" => "CurveType.Step",
                    "stepAfter" => "CurveType.StepAfter",
                    "stepBefore" => "CurveType.StepBefore",
                    _ => "CurveType.MonotoneX",
                };
                attrs.push(format!(":curve-type=\"{}\"", mapped));
            } else if let AuraPropValue::Expr(crate::ast::Expr::Ident(name)) = value {
                attrs.push(format!(":curve-type=\"{}\"", name));
            }
        }
    }

    /// Convert template string with ${...} placeholders to Vue {{ ... }} interpolation
    fn convert_template_to_vue(&self, template: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = template.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Look for ${ pattern
            if i + 1 < chars.len() && chars[i] == '$' && chars[i + 1] == '{' {
                // Find the closing }
                let start = i + 2;
                let mut depth = 1;
                let mut end = start;

                while end < chars.len() && depth > 0 {
                    if chars[end] == '{' {
                        depth += 1;
                    } else if chars[end] == '}' {
                        depth -= 1;
                    }
                    if depth > 0 {
                        end += 1;
                    }
                }

                if depth == 0 {
                    // Extract the expression inside ${...}
                    let expr: String = chars[start..end].iter().collect();
                    // Convert to Vue interpolation
                    let vue_expr = self.convert_template_expr_to_vue(&expr);
                    result.push_str(&format!("{{{{ {} }}}}", vue_expr));
                    i = end + 1;
                    continue;
                }
            }

            result.push(chars[i]);
            i += 1;
        }

        result
    }

    /// Convert a template expression (inside ${...}) to Vue expression
    fn convert_template_expr_to_vue(&self, expr: &str) -> String {
        let expr = expr.trim();

        // Handle state reference: .field -> field
        if expr.starts_with('.') {
            return expr[1..].to_string();
        }

        // Handle nested field access patterns like (dot (name user).name)
        // These come from the f-string parser's debug format
        if expr.starts_with('(') {
            return self.parse_s_expr_to_vue(expr);
        }

        // Handle simple field access: user.name
        if expr.contains('.') && !expr.starts_with('.') {
            return expr.to_string();
        }

        expr.to_string()
    }

    /// Parse S-expression format from f-string parser to Vue expression
    fn parse_s_expr_to_vue(&self, expr: &str) -> String {
        // Handle (dot (name user).field) -> user.field
        // Handle (dot (name user).id) -> user.id
        if let Some(inner) = expr.strip_prefix("(dot ") {
            // Find the object expression and the field
            // Format: (dot <object>.<field>)
            // Example: (dot (name user).id) means user.id

            // Find where the object ends and field begins
            // Look for the pattern: ).<field>)
            if let Some(dot_pos) = inner.rfind('.') {
                // Everything before the dot is the object expression
                let obj_expr = &inner[..dot_pos];
                // Everything after the dot (and before the final ')') is the field
                let field = inner[dot_pos + 1..].trim_end_matches(')').trim();

                // Parse the object expression
                let obj_name = if obj_expr.starts_with("(name ") {
                    // (name user) -> user
                    obj_expr[6..].trim().trim_end_matches(')').to_string()
                } else {
                    self.parse_s_expr_to_vue(obj_expr)
                };

                return format!("{}.{}", obj_name, field);
            }
        }

        // Handle (name user) -> user
        if let Some(inner) = expr.strip_prefix("(name ") {
            return inner.trim_end_matches(')').trim().to_string();
        }

        // Fallback: return as-is
        expr.to_string()
    }

    // ========================================================================
    // shadcn-vue Component-specific Prop Handling
    // ========================================================================

    /// Generate shadcn-vue component attributes based on element type
    /// Returns: (attributes, text_content, generated_children_html)
    fn generate_shadcn_attrs(
        &mut self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
    ) -> (Vec<String>, Option<String>, Option<String>) {
        let mut attrs = Vec::new();
        let mut slot_content: Option<String> = None;
        let mut slot_children: Option<String> = None;

        // Template ref escape hatch: `ref: "menuEl"` → static `ref="menuEl"`
        // attribute + a `const menuEl = ref<HTMLElement | null>(null)` in
        // <script setup> (from self.template_refs). Handled generically so
        // every shadcn-mapped element supports it without per-arm changes.
        if let Some(value) = props.get("ref") {
            let ref_name = match value {
                AuraPropValue::Expr(crate::ast::Expr::Str(name)) => name.to_string(),
                AuraPropValue::Expr(crate::ast::Expr::Ident(name)) => name.to_string(),
                _ => String::new(),
            };
            if !ref_name.is_empty() {
                if !self.template_refs.contains(&ref_name) {
                    self.template_refs.push(ref_name.clone());
                }
                attrs.push(format!("ref=\"{}\"", ref_name));
            }
        }

        // Inline style object: `style_obj: { top: f"${.y}px", "z-index": 50 }`
        // → `:style="{ top: `${y}px`, 'z-index': 50 }"`. Handled generically
        // (like `ref` above) so every shadcn-mapped element supports it.
        if let Some(AuraPropValue::StyleBinding(bindings)) = props.get("style_obj") {
            attrs.push(format!(":style=\"{}\"", self.style_obj_to_vue(bindings)));
        }

        // Normalize tag for matching (kebab-case -> snake_case, lowercase for case-insensitive matching)
        let normalized_tag = tag.replace('-', "_").to_lowercase();

        match normalized_tag.as_str() {
            // === Button ===
            "button" => {
                // Handle variant prop (default, secondary, destructive, outline, ghost, link)
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("variant=\"{}\"", variant));
                }
                // Handle size prop (sm, default, lg, icon)
                if let Some(value) = props.get("size") {
                    let size = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("size=\"{}\"", size));
                }
                // Handle disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // Handle style/class prop
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    if !class.is_empty() {
                        attrs.push(format!("class=\"{}\"", class));
                    }
                }
                // Build slot children for icon + text
                let mut button_children = Vec::new();
                if let Some(icon_name) = props.get("icon").and_then(|v| self.extract_string_value(v)) {
                    let lucide_component = Self::kebab_to_pascal(icon_name);
                    self.lucide_icons.insert(lucide_component.clone());
                    button_children.push(format!(r#"<{} class="h-4 w-4" />"#, lucide_component));
                }
                if let Some(value) = props.get("text") {
                    if let Ok(text) = self.prop_to_text_content(value) {
                        button_children.push(text);
                    }
                }
                if !button_children.is_empty() {
                    slot_children = Some(button_children.join(""));
                }
            }

            // === Layout Elements (Row, Col, Scroll, etc.) ===
            // These always need their structural flex classes, even when user provides style/class.
            // User classes are appended after the structural defaults (deduped to avoid repetition).
            "row" => {
                let mut classes = vec!["flex".to_string(), "flex-row".to_string()];
                if let Some(value) = self.get_style_class(props) {
                    let user_class = self.extract_string_value(value).unwrap_or("");
                    if !user_class.is_empty() {
                        for c in user_class.split_whitespace() {
                            if !classes.iter().any(|d| d == c) {
                                classes.push(c.to_string());
                            }
                        }
                    }
                }
                attrs.push(format!("class=\"{}\"", classes.join(" ")));
            }

            "col" | "column" => {
                let mut classes = vec!["flex".to_string(), "flex-col".to_string()];
                if let Some(value) = self.get_style_class(props) {
                    let user_class = self.extract_string_value(value).unwrap_or("");
                    if !user_class.is_empty() {
                        for c in user_class.split_whitespace() {
                            if !classes.iter().any(|d| d == c) {
                                classes.push(c.to_string());
                            }
                        }
                    }
                }
                attrs.push(format!("class=\"{}\"", classes.join(" ")));
            }

            "scroll" => {
                // ScrollArea support (Plan 105)
                // viewport class for styling
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
                // orientation (vertical, horizontal, both)
                if let Some(value) = props.get("orientation") {
                    let orientation = self.extract_string_value(value).unwrap_or("vertical");
                    attrs.push(format!("orientation=\"{}\"", orientation));
                }
                // scroll hide delay
                if let Some(value) = props.get("hide_delay") {
                    if let Some(delay) = self.extract_int_value(value) {
                        attrs.push(format!(":scroll-hide-delay=\"{}\"", delay));
                    }
                }
            }

            "container" => {
                // Default: max-w-7xl mx-auto + user style
                let mut classes = vec!["max-w-7xl".to_string(), "mx-auto".to_string()];
                if let Some(value) = self.get_style_class(props) {
                    let user_class = self.extract_string_value(value).unwrap_or("");
                    if !user_class.is_empty() {
                        classes.push(user_class.to_string());
                    }
                }
                attrs.push(format!("class=\"{}\"", classes.join(" ")));
            }

            "center" => {
                // Default: flex items-center justify-center h-full + user style
                // Note: do NOT add w-full here — if user has max-w-*, mx-auto handles centering.
                // If no max-width is specified, the element fills width naturally via flex.
                let mut classes = vec![
                    "flex".to_string(),
                    "flex-col".to_string(),
                    "items-center".to_string(),
                    "justify-center".to_string(),
                    "h-full".to_string(),
                ];
                if let Some(value) = self.get_style_class(props) {
                    let user_class = self.extract_string_value(value).unwrap_or("");
                    if !user_class.is_empty() {
                        classes.push(user_class.to_string());
                        // If user style has max-w-*, add mx-auto to center the constrained element
                        if user_class.contains("max-w-") {
                            classes.push("mx-auto".to_string());
                        }
                    }
                }
                attrs.push(format!("class=\"{}\"", classes.join(" ")));
            }

            "grid" => {
                let mut classes = vec!["grid".to_string()];
                // cols prop → grid-template-columns
                if let Some(value) = props.get("cols") {
                    if let Some(n) = self.extract_int_value(value) {
                        classes.push(format!("grid-cols-{}", n));
                    }
                }
                // gap prop
                if let Some(value) = props.get("gap") {
                    if let Some(n) = self.extract_int_value(value) {
                        classes.push(format!("gap-{}", n));
                    } else {
                        classes.push("gap-4".to_string());
                    }
                }
                if let Some(value) = self.get_style_class(props) {
                    let user_class = self.extract_string_value(value).unwrap_or("");
                    if !user_class.is_empty() {
                        classes.push(user_class.to_string());
                    }
                }
                attrs.push(format!("class=\"{}\"", classes.join(" ")));
            }

            "grid-item" | "grid_item" => {
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    if !class.is_empty() {
                        attrs.push(format!("class=\"{}\"", class));
                    }
                }
            }

            // === Link (Navigation Link) ===
            "link" => {
                // Text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // href (optional, if present use as link destination)
                if let Some(value) = props.get("href") {
                    let href = self.extract_string_value(value).unwrap_or("#");
                    attrs.push(format!("href=\"{}\"", href));
                }
            }

            // === Nav Link (Sidebar navigation item) ===
            "nav_link" => {
                if let Some(value) = props.get("to") {
                    let to = self.extract_string_value(value).unwrap_or("#");
                    attrs.push(format!("to=\"{}\"", to));
                }
                if let Some(label) = props.get("label").and_then(|v| self.extract_string_value(v)) {
                    let icon_name = props.get("icon").and_then(|v| self.extract_string_value(v));
                    if let Some(icon) = icon_name {
                        let lucide_component = Self::kebab_to_pascal(icon);
                        self.lucide_icons.insert(lucide_component.clone());
                        slot_children = Some(format!(
                            r#"<div class="flex flex-row items-center gap-2 rounded-md px-2 py-1.5 text-sm"><{} class="h-4 w-4 shrink-0" /><span>{}</span></div>"#,
                            lucide_component, label
                        ));
                    } else {
                        slot_children = Some(format!(r#"<span>{}</span>"#, label));
                    }
                }
            }

            // === CodeBlock ===
            "codeblock" => {
                // lang prop for language identifier
                if let Some(value) = props.get("lang") {
                    let lang = self.extract_string_value(value).unwrap_or("text");
                    attrs.push(format!("data-lang=\"{}\"", lang));
                }
                // code content becomes slot content
                if let Some(value) = props.get("code") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === PreviewCard ===
            "previewcard" | "preview-card" => {
                // title prop (default: "Preview")
                let title = if let Some(value) = props.get("title") {
                    self.extract_string_value(value).unwrap_or("Preview").to_string()
                } else {
                    "Preview".to_string()
                };
                // auto and vue props are stored as data attributes for the code section
                if let Some(value) = props.get("auto") {
                    if let Some(auto_code) = self.extract_string_value(value) {
                        attrs.push(format!("data-auto=\"{}\"", auto_code.replace("\"", "&quot;").replace("<", "&lt;")));
                    }
                }
                if let Some(value) = props.get("vue") {
                    if let Some(vue_code) = self.extract_string_value(value) {
                        attrs.push(format!("data-vue=\"{}\"", vue_code.replace("\"", "&quot;").replace("<", "&lt;")));
                    }
                }
                let _ = title; // Suppress unused variable warning
            }

            // === AutoDownEditor (Plan 354 Phase C) ===
            // Rich AutoDown WYSIWYG editor (Tiptap) consumed from @autodown/editor.
            // Maps snake_case AURA props to the wrapper's camelCase Vue props.
            "autodown_editor" | "autodowneditor" => {
                // content → :content (bound to the note body markdown).
                // Supports state refs (content: .body → body) and field access
                // (content: .note.body → note.body).
                match props.get("content") {
                    Some(AuraPropValue::Expr(expr)) => {
                        if let Ok(js_expr) = self.expr_to_vue_bound_value(expr) {
                            attrs.push(format!(":content=\"{}\"", js_expr));
                        }
                    }
                    Some(value) => {
                        // Literal string content
                        let content = self.extract_string_value(value).unwrap_or("");
                        attrs.push(format!("content=\"{}\"", content));
                    }
                    None => {}
                }

                // can_edit → :canEdit (bool). Defaults to true when omitted so
                // the editor is interactive by default.
                attrs.push(self.bool_prop_binding(props, "can_edit", "canEdit", true));

                // show_actions → :showActions (bool). Defaults to true.
                attrs.push(self.bool_prop_binding(props, "show_actions", "showActions", true));

                // style/class (editor chrome sizing).
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    if !class.is_empty() {
                        attrs.push(format!("class=\"{}\"", class));
                    }
                }
                // NOTE: events (onupdate/onsave/oncancel → @update/@save/@cancel)
                // are attached by the generic event loop at the end of this fn.
            }

            // === Input ===
            "input" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        // Prop-backed value (v-model contract child widget):
                        // props are read-only, so emit one-way :modelValue and
                        // let the event handler emit update:modelValue upward.
                        if self.prop_names.iter().any(|p| p == &model) {
                            attrs.push(format!(":modelValue=\"{}\"", model));
                        } else {
                            attrs.push(format!("v-model=\"{}\"", model));
                        }
                    }
                }
                // type prop
                if let Some(value) = props.get("type") {
                    let type_val = self.extract_string_value(value).unwrap_or("text");
                    attrs.push(format!("type=\"{}\"", type_val));
                }
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // style/class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    if !class.is_empty() {
                        attrs.push(format!("class=\"{}\"", class));
                    }
                }
            }

            // === Label ===
            "label" => {
                // for prop (link to input id)
                if let Some(value) = props.get("for") {
                    let for_val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("for=\"{}\"", for_val));
                }
                // Text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Text (Typography) ===
            "text" | "Text" | "span" | "Span" | "p" | "P" => {
                // Extract class/style for Tailwind
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    if !class.is_empty() {
                        attrs.push(format!("class=\"{}\"", class));
                    }
                }
                // Text content becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Headings (Typography) ===
            "h1" | "H1" | "h2" | "H2" | "h3" | "H3" | "h4" | "H4" | "h5" | "H5" | "h6" | "H6" => {
                // Extract class/style for Tailwind
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    if !class.is_empty() {
                        attrs.push(format!("class=\"{}\"", class));
                    }
                }
                // Text content becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Textarea ===
            "textarea" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        // Prop-backed value (v-model contract child widget):
                        // one-way :modelValue, update goes out via the handler.
                        if self.prop_names.iter().any(|p| p == &model) {
                            attrs.push(format!(":modelValue=\"{}\"", model));
                        } else {
                            attrs.push(format!("v-model=\"{}\"", model));
                        }
                    }
                }
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
                // rows
                if let Some(value) = props.get("rows") {
                    let rows = self.extract_int_value(value).unwrap_or(3);
                    attrs.push(format!(":rows=\"{}\"", rows));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // style/class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    if !class.is_empty() {
                        attrs.push(format!("class=\"{}\"", class));
                    }
                }
            }

            // === Checkbox ===
            "checkbox" => {
                // reka-ui CheckboxRoot uses modelValue (not checked), so use v-model / :model-value
                if let Some(value) = props.get("checked") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    } else if self.extract_bool_value(value) {
                        // Static true value - use :model-value for controlled mode
                        attrs.push(":model-value=\"true\"".to_string());
                    } else if let AuraPropValue::Expr(expr) = value {
                        // Dynamic expression (e.g., todo.done) — one-way :model-value binding
                        if let Ok(js_expr) = self.expr_to_vue_bound_value(expr) {
                            attrs.push(format!(":model-value=\"{}\"", js_expr));
                        }
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === Switch/Toggle ===
            "toggle" | "switch" => {
                // v-model:checked for checked state (dynamic) or :default-checked (static)
                if let Some(value) = props.get("checked") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:checked=\"{}\"", model));
                    } else if self.extract_bool_value(value) {
                        // Static true value - use default-checked for uncontrolled mode
                        attrs.push(":default-checked=\"true\"".to_string());
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === Select ===
            "select" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === SelectItem ===
            "selectitem" | "select_item" => {
                // value for selection
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === SelectValue ===
            "selectvalue" | "select_value" => {
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
            }

            // === SelectTrigger ===
            "selecttrigger" | "select_trigger" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === SelectLabel ===
            "selectlabel" | "select_label" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Slider ===
            "slider" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // min/max/step
                if let Some(value) = props.get("min") {
                    let min = self.extract_int_value(value).unwrap_or(0);
                    attrs.push(format!(":min=\"{}\"", min));
                }
                if let Some(value) = props.get("max") {
                    let max = self.extract_int_value(value).unwrap_or(100);
                    attrs.push(format!(":max=\"{}\"", max));
                }
                if let Some(value) = props.get("step") {
                    let step = self.extract_int_value(value).unwrap_or(1);
                    attrs.push(format!(":step=\"{}\"", step));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === Progress ===
            "progress" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    } else if let Some(int_val) = self.extract_int_value(value) {
                        attrs.push(format!(":model-value=\"{}\"", int_val));
                    }
                }
                // max
                if let Some(value) = props.get("max") {
                    let max = self.extract_int_value(value).unwrap_or(100);
                    attrs.push(format!(":max=\"{}\"", max));
                }
            }

            // === Badge ===
            "badge" => {
                // variant
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("variant=\"{}\"", variant));
                }
                // Handle style/class prop
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    if !class.is_empty() {
                        attrs.push(format!("class=\"{}\"", class));
                    }
                }
                // Text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Card ===
            "card" => {
                // Card variant (default, outline, ghost)
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("variant=\"{}\"", variant));
                }
                // Card title becomes header
                if let Some(value) = props.get("title") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }


            // === Tabs ===
            "tabs" => {
                // v-model for active tab value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    } else if let Some(val) = self.extract_string_value(value) {
                        attrs.push(format!("default-value=\"{}\"", val));
                    }
                }
                // default value
                if let Some(value) = props.get("default") {
                    let default = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("default-value=\"{}\"", default));
                }
            }

            // === Tab ===
            "tab" => {
                // Tab value (required for TabsTrigger)
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // Disabled state
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // Text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Separator/Divider ===
            "divider" | "separator" => {
                // orientation (horizontal, vertical)
                if let Some(value) = props.get("orientation") {
                    let orientation = self.extract_string_value(value).unwrap_or("horizontal");
                    attrs.push(format!("orientation=\"{}\"", orientation));
                }
                // decorative (accessibility)
                if let Some(value) = props.get("decorative") {
                    if self.extract_bool_value(value) {
                        attrs.push("decorative".to_string());
                    }
                }
                // label for accessibility
                if let Some(value) = props.get("label") {
                    let label = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("label=\"{}\"", label));
                }
            }

            // === AlertDialog Sub-components ===
            "alertdialog" | "alert_dialog" => {
                // v-model:open for dialog state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
            }
            "alertdialogtrigger" | "alert-dialog-trigger" => {
                // text becomes slot content, as-child for button styling
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                if let Some(value) = props.get("asChild") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
            }
            "alertdialogcontent" | "alert-dialog-content" => {
                // class for styling
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }
            "alertdialogheader" | "alert-dialog-header" | "alertdialogfooter" | "alert-dialog-footer" => {
                // Container components - class handled by extract_classes
            }
            "alertdialogtitle" | "alert-dialog-title" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }
            "alertdialogdescription" | "alert-dialog-description" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }
            "alertdialogaction" | "alert-dialog-action" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }
            "alertdialogcancel" | "alert-dialog-cancel" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Dialog Sub-components ===
            "dialogtrigger" | "dialog_trigger" | "dialog-trigger" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }
            "dialogcontent" | "dialog_content" | "dialog-content" => {
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }
            "dialogheader" | "dialog_header" | "dialog-header" | "dialogfooter" | "dialog_footer" | "dialog-footer" => {
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }
            "dialogtitle" | "dialog_title" | "dialog-title" => {
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }
            "dialogdescription" | "dialog_description" | "dialog-description" => {
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }
            "dialogclose" | "dialog_close" | "dialog-close" => {
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Card Sub-components ===
            "cardheader" | "cardcontent" | "cardfooter" => {
                // These are container components - class is handled by extract_classes
            }
            "cardtitle" | "carddescription" => {
                // Text content for title/description
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Tabs Sub-components ===
            "tabslist" | "tabs_list" => {
                // TabsList is a container - class handled by extract_classes
            }
            "tabstrigger" | "tabs_trigger" => {
                // value is required for TabsTrigger
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // Text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }
            "tabscontent" | "tabs_content" => {
                // value is required for TabsContent
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
            }

            // === Avatar ===
            "avatar" => {
                // Avatar in shadcn-vue is a wrapper that needs AvatarImage and AvatarFallback children
                let mut generated_children = String::new();

                // Generate AvatarImage if src provided
                if let Some(value) = props.get("src") {
                    let src = self.extract_string_value(value).unwrap_or("");
                    let alt = props.get("alt")
                        .and_then(|v| self.extract_string_value(v))
                        .unwrap_or("");
                    generated_children.push_str(&format!(
                        r#"<AvatarImage src="{}" alt="{}" />{}"#,
                        src, alt, "\n"
                    ));
                    // Register AvatarImage component for imports
                    self.shadcn_components_used.insert("AvatarImage".to_string());
                }

                // Generate AvatarFallback if fallback provided
                if let Some(value) = props.get("fallback") {
                    let fallback_text = self.prop_to_text_content(value).unwrap_or_default();
                    generated_children.push_str(&format!(
                        r#"<AvatarFallback>{}</AvatarFallback>"#,
                        fallback_text
                    ));
                    // Register AvatarFallback component for imports
                    self.shadcn_components_used.insert("AvatarFallback".to_string());
                }

                // Set generated children if any were created
                if !generated_children.is_empty() {
                    slot_children = Some(generated_children);
                }
            }

            // === AvatarImage (when used as standalone element) ===
            "avatarimage" | "avatar_image" => {
                if let Some(value) = props.get("src") {
                    let src = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("src=\"{}\"", src));
                }
                if let Some(value) = props.get("alt") {
                    let alt = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("alt=\"{}\"", alt));
                }
            }

            // === AvatarFallback (when used as standalone element) ===
            "avatarfallback" | "avatar_fallback" => {
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === AspectRatio ===
            "aspectratio" | "aspect_ratio" => {
                // ratio prop (e.g., 16/9 = 1.777)
                if let Some(value) = props.get("ratio") {
                    if let Some(ratio) = self.extract_float_value(value) {
                        attrs.push(format!(":ratio=\"{}\"", ratio));
                    } else if let Some(ratio) = self.extract_int_value(value) {
                        attrs.push(format!(":ratio=\"{}\"", ratio));
                    }
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // ========================================
            // Phase 4: Overlay & Feedback
            // ========================================

            // === Dialog/Modal ===
            "dialog" | "modal" => {
                // v-model:open for dialog state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
                // title for DialogTitle
                if let Some(value) = props.get("title") {
                    let title = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("data-title=\"{}\"", title));
                }
                // description for DialogDescription
                if let Some(value) = props.get("description") {
                    let desc = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("data-description=\"{}\"", desc));
                }
            }

            // === Tooltip ===
            "tooltip" => {
                // content for TooltipContent
                if let Some(value) = props.get("content") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // side (top, right, bottom, left)
                if let Some(value) = props.get("side") {
                    let side = self.extract_string_value(value).unwrap_or("top");
                    attrs.push(format!("side=\"{}\"", side));
                }
                // delay duration
                if let Some(value) = props.get("delay") {
                    if let Some(delay) = self.extract_int_value(value) {
                        attrs.push(format!(":delay-duration=\"{}\"", delay));
                    }
                }
            }

            // === Spinner/Skeleton ===
            "spinner" => {
                // Skeleton uses class for sizing
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
                // width
                if let Some(value) = props.get("width") {
                    if let Some(width) = self.extract_int_value(value) {
                        attrs.push(format!("style=\"width: {}px\"", width));
                    }
                }
                // height
                if let Some(value) = props.get("height") {
                    if let Some(height) = self.extract_int_value(value) {
                        attrs.push(format!("style=\"height: {}px\"", height));
                    }
                }
            }

            // ========================================
            // Phase 5: Data Components
            // ========================================

            // === Table ===
            "table" => {
                // Table wrapper class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "thead" | "tbody" | "tr" => {
                // Table structure elements - minimal props
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "th" | "td" => {
                // Table cells
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
                // colspan
                if let Some(value) = props.get("colspan") {
                    if let Some(span) = self.extract_int_value(value) {
                        attrs.push(format!(":colspan=\"{}\"", span));
                    }
                }
                // rowspan
                if let Some(value) = props.get("rowspan") {
                    if let Some(span) = self.extract_int_value(value) {
                        attrs.push(format!(":rowspan=\"{}\"", span));
                    }
                }
                // Text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === shadcn-vue Table components ===
            "table_caption" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "table_header" | "table_body" | "table_row" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "table_head" | "table_cell" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Tree ===
            "tree" => {
                // Tree container
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "tree_item" | "tree-item" => {
                // Tree item with expanded state
                if let Some(value) = props.get("expanded") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
                // Text content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // ========================================
            // Phase 6: Form Components
            // ========================================

            // === RadioGroup ===
            "radiogroup" | "radio-group" => {
                // v-model for selected value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // name for form grouping
                if let Some(value) = props.get("name") {
                    let name = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("name=\"{}\"", name));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === Radio ===
            "radio" => {
                // value for this radio option
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // id for label association
                if let Some(value) = props.get("id") {
                    let id = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("id=\"{}\"", id));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // label text
                if let Some(value) = props.get("label") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // ========================================
            // Phase 7: Feedback & Navigation
            // ========================================

            // === Alert ===
            "alert" => {
                // variant: default, destructive
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("variant=\"{}\"", variant));
                }
                // title for AlertTitle
                if let Some(value) = props.get("title") {
                    let title = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("data-title=\"{}\"", title));
                }
                // description/text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                if let Some(value) = props.get("description") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Toast/Toaster (Sonner) ===
            "toast" | "toaster" => {
                // position: top-left, top-center, top-right, bottom-left, bottom-center, bottom-right
                if let Some(value) = props.get("position") {
                    let position = self.extract_string_value(value).unwrap_or("bottom-right");
                    attrs.push(format!("position=\"{}\"", position));
                }
                // richColors for colored toasts
                if let Some(value) = props.get("rich_colors") {
                    if self.extract_bool_value(value) {
                        attrs.push(":rich-colors=\"true\"".to_string());
                    }
                }
                // expand for expanded toasts
                if let Some(value) = props.get("expand") {
                    if self.extract_bool_value(value) {
                        attrs.push(":expand=\"true\"".to_string());
                    }
                }
            }

            // === Dropdown Menu ===
            "dropdown" | "dropdown_menu" => {
                // v-model:open for menu state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
            }

            "dropdown_trigger" => {
                // as-child for custom trigger
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "dropdown_content" => {
                // side: top, right, bottom, left
                if let Some(value) = props.get("side") {
                    let side = self.extract_string_value(value).unwrap_or("bottom");
                    attrs.push(format!("side=\"{}\"", side));
                }
                // align: start, center, end
                if let Some(value) = props.get("align") {
                    let align = self.extract_string_value(value).unwrap_or("center");
                    attrs.push(format!("align=\"{}\"", align));
                }
            }

            "dropdown_item" => {
                // value for selection
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "dropdown_separator" => {
                // No special attributes
            }

            "dropdown_label" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // ========================================
            // Phase 8: Popover, Sheet, Breadcrumb
            // ========================================

            // === Popover ===
            "popover" => {
                // v-model:open for popover state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
            }

            "popover_trigger" => {
                // as-child for custom trigger
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "popover_content" => {
                // side: top, right, bottom, left
                if let Some(value) = props.get("side") {
                    let side = self.extract_string_value(value).unwrap_or("bottom");
                    attrs.push(format!("side=\"{}\"", side));
                }
                // align: start, center, end
                if let Some(value) = props.get("align") {
                    let align = self.extract_string_value(value).unwrap_or("center");
                    attrs.push(format!("align=\"{}\"", align));
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Sheet (Side Drawer) ===
            "sheet" => {
                // v-model:open for sheet state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
            }

            "sheet_trigger" => {
                // as-child for custom trigger
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "sheet_content" => {
                // side: top, right, bottom, left
                if let Some(value) = props.get("side") {
                    let side = self.extract_string_value(value).unwrap_or("right");
                    attrs.push(format!("side=\"{}\"", side));
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "sheet_header" | "sheet_footer" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "sheet_title" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Breadcrumb ===
            "breadcrumb" | "breadcrumb_list" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "breadcrumb_item" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "breadcrumb_link" => {
                // href for link
                if let Some(value) = props.get("href") {
                    let href = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("href=\"{}\"", href));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick for navigation
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "breadcrumb_separator" => {
                // No special attributes
            }

            "breadcrumb_page" => {
                // text becomes slot content (current page, not clickable)
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // ========================================
            // Phase 9: High Priority Components
            // ========================================

            // === Accordion ===
            "accordion" => {
                // type: single, multiple
                if let Some(value) = props.get("type") {
                    let type_val = self.extract_string_value(value).unwrap_or("single");
                    attrs.push(format!("type=\"{}\"", type_val));
                }
                // collapsible (for single type)
                if let Some(value) = props.get("collapsible") {
                    if self.extract_bool_value(value) {
                        attrs.push(":collapsible=\"true\"".to_string());
                    }
                }
                // default-value for initially expanded item
                if let Some(value) = props.get("default") {
                    let default = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("default-value=\"{}\"", default));
                }
            }

            "accordion_item" => {
                // value (required)
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
            }

            "accordion_trigger" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "accordion_content" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "alert_dialog_trigger" => {
                // as-child for custom trigger
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
            }

            "alert_dialog_content" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "alert_dialog_header" | "alert_dialog_footer" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "alert_dialog_title" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "alert_dialog_description" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "alert_dialog_action" | "alert_dialog_cancel" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick event
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            // === Command (Command Palette) ===
            "command" => {
                // v-model for search query
                if let Some(value) = props.get("query") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:search-term=\"{}\"", model));
                    }
                }
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("Type a command or search...");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
            }

            "command_input" => {
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("Type a command...");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
            }

            "command_list" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "command_empty" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "command_group" => {
                // heading
                if let Some(value) = props.get("heading") {
                    let heading = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("heading=\"{}\"", heading));
                }
            }

            "command_item" => {
                // value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick for selection
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "command_shortcut" => {
                // text becomes slot content (e.g., "⌘K")
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "command_separator" => {
                // No special attributes
            }

            // === Form ===
            "form" => {
                // id for form identification
                if let Some(value) = props.get("id") {
                    let id = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("id=\"{}\"", id));
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
                // onsubmit event
                if events.contains_key("onsubmit") {
                    // Handled by event handlers below
                }
            }

            "form_field" => {
                // name (required)
                if let Some(value) = props.get("name") {
                    let name = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("name=\"{}\"", name));
                }
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
            }

            "form_item" | "form_control" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "form_label" => {
                // for (htmlFor)
                if let Some(value) = props.get("for") {
                    let for_val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("for=\"{}\"", for_val));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "form_description" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "form_message" => {
                // Error message (auto-bound to form validation)
            }

            // === Navigation Menu ===
            "nav_menu" => {
                // orientation: horizontal, vertical
                if let Some(value) = props.get("orientation") {
                    let orientation = self.extract_string_value(value).unwrap_or("horizontal");
                    attrs.push(format!("orientation=\"{}\"", orientation));
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "nav_menu_list" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "nav_menu_item" => {
                // value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
            }

            "nav_menu_link" => {
                // href
                if let Some(value) = props.get("href") {
                    let href = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("href=\"{}\"", href));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "nav_menu_trigger" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "nav_menu_content" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "nav_menu_indicator" => {
                // No special attributes
            }

            // === Sidebar ===
            "sidebar" => {
                // side: left, right
                if let Some(value) = props.get("side") {
                    let side = self.extract_string_value(value).unwrap_or("left");
                    attrs.push(format!("side=\"{}\"", side));
                }
                // variant: sidebar, floating, inset
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("sidebar");
                    attrs.push(format!("variant=\"{}\"", variant));
                }
                // collapsible: offcanvas, icon, none
                if let Some(value) = props.get("collapsible") {
                    let collapsible = self.extract_string_value(value).unwrap_or("offcanvas");
                    attrs.push(format!("collapsible=\"{}\"", collapsible));
                }
            }

            "sidebar_header" | "sidebar_footer" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "sidebar_content" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "sidebar_group" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "sidebar_group_label" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "sidebar_group_content" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "sidebar_menu" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "sidebar_menu_item" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "sidebar_menu_button" => {
                // tooltip
                if let Some(value) = props.get("tooltip") {
                    let tooltip = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("tooltip=\"{}\"", tooltip));
                }
                // isActive
                if let Some(value) = props.get("active") {
                    if self.extract_bool_value(value) {
                        attrs.push(":is-active=\"true\"".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "sidebar_trigger" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "sidebar_provider" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Stepper ===
            "stepper" => {
                // v-model for current step
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // orientation: horizontal, vertical
                if let Some(value) = props.get("orientation") {
                    let orientation = self.extract_string_value(value).unwrap_or("horizontal");
                    attrs.push(format!("orientation=\"{}\"", orientation));
                }
            }

            "stepper_item" => {
                // step (required)
                if let Some(value) = props.get("step") {
                    if let Some(step) = self.extract_int_value(value) {
                        attrs.push(format!(":step=\"{}\"", step));
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            "stepper_trigger" => {
                // onclick for step navigation
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "stepper_indicator" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "stepper_title" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "stepper_description" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "stepper_separator" => {
                // No special attributes
            }

            // ========================================
            // Phase 10: Medium Priority Components
            // ========================================

            // === Calendar ===
            "calendar" => {
                // v-model for selected date
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // default-placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("Pick a date");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
                // weekday-format
                if let Some(value) = props.get("weekday") {
                    let weekday = self.extract_string_value(value).unwrap_or("short");
                    attrs.push(format!("weekday-format=\"{}\"", weekday));
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Carousel ===
            "carousel" => {
                // Build opts object for Embla options (align, loop, etc.)
                let mut opts_parts: Vec<String> = Vec::new();

                // align option
                if let Some(value) = props.get("align") {
                    let align = self.extract_string_value(value).unwrap_or("center");
                    opts_parts.push(format!("align: '{}'", align));
                }

                // loop option
                if let Some(value) = props.get("loop") {
                    if self.extract_bool_value(value) {
                        opts_parts.push("loop: true".to_string());
                    }
                }

                // orientation option (vertical/horizontal)
                // This is a direct prop on Carousel for shadcn-vue styling
                if let Some(value) = props.get("orientation") {
                    let orientation = self.extract_string_value(value).unwrap_or("horizontal");
                    attrs.push(format!("orientation=\"{}\"", orientation));
                }

                // Output opts if any options were specified
                if !opts_parts.is_empty() {
                    attrs.push(format!(":opts=\"{{ {} }}\"", opts_parts.join(", ")));
                }

                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "carousel_content" | "carousel_item" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "carousel_prev" | "carousel_previous" | "carousel_next" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Combobox ===
            "combobox" => {
                // v-model for selected value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // open state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
            }

            "combobox_input" => {
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("Select...");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
            }

            "combobox_item" => {
                // value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick for selection
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "combobox_trigger" => {
                // as-child
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
            }

            "combobox_empty" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Context Menu ===
            "context_menu" => {
                // v-model:open for menu state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
            }

            "context_menu_trigger" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
                // as-child for custom trigger
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
            }

            "context_menu_content" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "context_menu_item" => {
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // inset
                if let Some(value) = props.get("inset") {
                    if self.extract_bool_value(value) {
                        attrs.push("inset".to_string());
                    }
                }
                // variant (default, destructive)
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    if variant != "default" {
                        attrs.push(format!("variant=\"{}\"", variant));
                    }
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // shortcut - rendered as ContextMenuShortcut inside the item
                // (handled in node_to_html when children are processed)
                // onclick
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "context_menu_separator" => {
                // No special attributes
            }

            "context_menu_label" => {
                // inset
                if let Some(value) = props.get("inset") {
                    if self.extract_bool_value(value) {
                        attrs.push("inset".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "context_menu_shortcut" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "context_menu_checkbox_item" => {
                // model-value for checked state
                if let Some(value) = props.get("checked") {
                    if self.extract_bool_value(value) {
                        attrs.push(":model-value=\"true\"".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "context_menu_radio_group" => {
                // model-value for selected value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("model-value=\"{}\"", val));
                }
            }

            "context_menu_radio_item" => {
                // value (required)
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "context_menu_sub" => {
                // open
                if let Some(value) = props.get("open") {
                    if let Some(ref_name) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", ref_name));
                    }
                }
            }

            "context_menu_sub_trigger" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // inset
                if let Some(value) = props.get("inset") {
                    if self.extract_bool_value(value) {
                        attrs.push("inset".to_string());
                    }
                }
            }

            "context_menu_sub_content" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Drawer (Vaul) ===
            "drawer" => {
                // v-model:open for drawer state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
                // direction: left, right, top, bottom
                if let Some(value) = props.get("direction") {
                    let direction = self.extract_string_value(value).unwrap_or("bottom");
                    attrs.push(format!("direction=\"{}\"", direction));
                }
            }

            "drawer_trigger" => {
                // as-child for custom trigger
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "drawer_content" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "drawer_header" | "drawer_footer" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "drawer_title" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "drawer_description" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "drawer_close" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            // === Hover Card ===
            "hover_card" => {
                // v-model:open for hover card state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
                // open-delay
                if let Some(value) = props.get("open_delay") {
                    if let Some(delay) = self.extract_int_value(value) {
                        attrs.push(format!(":open-delay=\"{}\"", delay));
                    }
                }
                // close-delay
                if let Some(value) = props.get("close_delay") {
                    if let Some(delay) = self.extract_int_value(value) {
                        attrs.push(format!(":close-delay=\"{}\"", delay));
                    }
                }
            }

            "hover_card_trigger" => {
                // as-child for custom trigger
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
            }

            "hover_card_content" => {
                // side
                if let Some(value) = props.get("side") {
                    let side = self.extract_string_value(value).unwrap_or("bottom");
                    attrs.push(format!("side=\"{}\"", side));
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Number Field ===
            "number_field" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // min/max/step
                if let Some(value) = props.get("min") {
                    if let Some(min) = self.extract_int_value(value) {
                        attrs.push(format!(":min=\"{}\"", min));
                    }
                }
                if let Some(value) = props.get("max") {
                    if let Some(max) = self.extract_int_value(value) {
                        attrs.push(format!(":max=\"{}\"", max));
                    }
                }
                if let Some(value) = props.get("step") {
                    if let Some(step) = self.extract_int_value(value) {
                        attrs.push(format!(":step=\"{}\"", step));
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            "number_field_input" => {
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
            }

            "number_field_increment" | "number_field_decrement" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Pagination ===
            "pagination" => {
                // v-model:page for current page
                if let Some(value) = props.get("page") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:page=\"{}\"", model));
                    }
                }
                // total
                if let Some(value) = props.get("total") {
                    if let Some(total) = self.extract_int_value(value) {
                        attrs.push(format!(":total=\"{}\"", total));
                    }
                }
                // per-page / items-per-page
                if let Some(value) = props.get("per_page") {
                    if let Some(per_page) = self.extract_int_value(value) {
                        attrs.push(format!(":items-per-page=\"{}\"", per_page));
                    }
                }
                if let Some(value) = props.get("itemsPerPage") {
                    if let Some(items) = self.extract_int_value(value) {
                        attrs.push(format!(":items-per-page=\"{}\"", items));
                    }
                }
                // sibling-count
                if let Some(value) = props.get("sibling_count") {
                    if let Some(count) = self.extract_int_value(value) {
                        attrs.push(format!(":sibling-count=\"{}\"", count));
                    }
                }
            }

            "pagination_list" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "pagination_item" => {
                // value (page number)
                if let Some(value) = props.get("value") {
                    if let Some(val) = self.extract_int_value(value) {
                        attrs.push(format!(":value=\"{}\"", val));
                    }
                }
                // onclick for page change
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "pagination_ellipsis" => {
                // No special attributes
            }

            "pagination_prev" | "pagination_next" | "pagination_first" | "pagination_last" => {
                // onclick
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            // === Pin Input (OTP) ===
            "pin_input" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // length (number of pins)
                if let Some(value) = props.get("length") {
                    if let Some(length) = self.extract_int_value(value) {
                        attrs.push(format!(":length=\"{}\"", length));
                    }
                }
                // type: text, password
                if let Some(value) = props.get("type") {
                    let type_val = self.extract_string_value(value).unwrap_or("text");
                    attrs.push(format!("type=\"{}\"", type_val));
                }
                // otp (native autocomplete)
                if let Some(value) = props.get("otp") {
                    if self.extract_bool_value(value) {
                        attrs.push("otp".to_string());
                    }
                }
            }

            "pin_input_slot" => {
                // index
                if let Some(value) = props.get("index") {
                    if let Some(index) = self.extract_int_value(value) {
                        attrs.push(format!(":index=\"{}\"", index));
                    }
                }
            }

            "pin_input_separator" => {
                // No special attributes
            }

            // === Tags Input ===
            "tags_input" => {
                // v-model for tags array
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("Add tag...");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
                // max-tags
                if let Some(value) = props.get("max") {
                    if let Some(max) = self.extract_int_value(value) {
                        attrs.push(format!(":max-tags=\"{}\"", max));
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            "tags_input_field" => {
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
            }

            "tags_input_item" => {
                // value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
            }

            "tags_input_delete" => {
                // onclick to remove tag
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            // === Toggle Group ===
            "toggle_group" => {
                // v-model for selected value(s)
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // type: single, multiple
                if let Some(value) = props.get("type") {
                    let type_val = self.extract_string_value(value).unwrap_or("single");
                    attrs.push(format!("type=\"{}\"", type_val));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            "toggle_group_item" => {
                // value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // aria-label
                if let Some(value) = props.get("label") {
                    let label = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("aria-label=\"{}\"", label));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // ========================================
            // Phase 11: Low Priority Components
            // ========================================

            // === Button Group ===
            "button_group" => {
                // orientation: horizontal, vertical
                if let Some(value) = props.get("orientation") {
                    let orientation = self.extract_string_value(value).unwrap_or("horizontal");
                    attrs.push(format!("orientation=\"{}\"", orientation));
                }
                // size
                if let Some(value) = props.get("size") {
                    let size = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("size=\"{}\"", size));
                }
                // variant
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("variant=\"{}\"", variant));
                }
            }

            // === Chart ===
            "chart" => {
                // config for chart styling
                if let Some(value) = props.get("config") {
                    let config = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!(":config=\"{}\"", config));
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Collapsible ===
            "collapsible" => {
                // v-model:open for expanded state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
                // default-open
                if let Some(value) = props.get("default_open") {
                    if self.extract_bool_value(value) {
                        attrs.push(":default-open=\"true\"".to_string());
                    }
                }
            }

            "collapsible_trigger" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
                // as-child
                if let Some(value) = props.get("as_child") {
                    if self.extract_bool_value(value) {
                        attrs.push("as-child".to_string());
                    }
                }
            }

            "collapsible_content" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Input Group ===
            "input_group" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Input OTP ===
            "input_otp" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // length
                if let Some(value) = props.get("length") {
                    if let Some(length) = self.extract_int_value(value) {
                        attrs.push(format!(":length=\"{}\"", length));
                    }
                }
                // pattern
                if let Some(value) = props.get("pattern") {
                    let pattern = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("pattern=\"{}\"", pattern));
                }
            }

            // === Kbd (Keyboard) ===
            "kbd" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Menubar ===
            "menubar" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "menubar_menu" => {
                // value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
            }

            "menubar_trigger" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            "menubar_content" => {
                // align
                if let Some(value) = props.get("align") {
                    let align = self.extract_string_value(value).unwrap_or("start");
                    attrs.push(format!("align=\"{}\"", align));
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "menubar_item" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // onclick
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "menubar_separator" => {
                // No special attributes
            }

            "menubar_label" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Native Select ===
            "native_select" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // name
                if let Some(value) = props.get("name") {
                    let name = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("name=\"{}\"", name));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Range Calendar ===
            "range_calendar" => {
                // v-model for date range
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("Pick a date range");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Resizable ===
            "resizable" | "resizable_panel_group" => {
                // direction: horizontal, vertical
                if let Some(value) = props.get("direction") {
                    let direction = self.extract_string_value(value).unwrap_or("horizontal");
                    attrs.push(format!("direction=\"{}\"", direction));
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "resizable_panel" => {
                // default-size
                if let Some(value) = props.get("default_size") {
                    if let Some(size) = self.extract_int_value(value) {
                        attrs.push(format!(":default-size=\"{}\"", size));
                    }
                }
                // min-size
                if let Some(value) = props.get("min_size") {
                    if let Some(size) = self.extract_int_value(value) {
                        attrs.push(format!(":min-size=\"{}\"", size));
                    }
                }
                // max-size
                if let Some(value) = props.get("max_size") {
                    if let Some(size) = self.extract_int_value(value) {
                        attrs.push(format!(":max-size=\"{}\"", size));
                    }
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "resizable_handle" => {
                // with-handle (show drag handle)
                if let Some(value) = props.get("with_handle") {
                    if self.extract_bool_value(value) {
                        attrs.push(":with-handle=\"true\"".to_string());
                    }
                }
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Auto Complete ===
            "autocomplete" => {
                // v-model for selected value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // open state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
                }
            }

            "autocomplete_input" => {
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("Search...");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
            }

            "autocomplete_item" => {
                // value
                if let Some(value) = props.get("value") {
                    let val = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("value=\"{}\"", val));
                }
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
                // onclick
                if events.contains_key("onclick") {
                    // Handled by event handlers below
                }
            }

            "autocomplete_list" => {
                // class
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "autocomplete_empty" => {
                // text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Image ===
            "image" | "img" => {
                for key in &["src", "alt"] {
                    if let Some(value) = props.get(*key) {
                        match value {
                            AuraPropValue::Expr(crate::ast::Expr::Ident(name)) => {
                                attrs.push(format!(":{}=\"{}\"", key, name));
                            }
                            AuraPropValue::Expr(crate::ast::Expr::Dot(..)) => {
                                if let Ok(val) = self.prop_to_attr_value(value) {
                                    attrs.push(format!(":{}={}", key, val));
                                }
                            }
                            _ => {
                                if let Ok(val) = self.prop_to_attr_value(value) {
                                    attrs.push(format!("{}={}", key, val));
                                }
                            }
                        }
                    }
                }
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    if !class.is_empty() {
                        attrs.push(format!("class=\"{}\"", class));
                    }
                }
            }

            // === Charts (shadcn-vue + Unovis) ===
            "area_chart" | "area-chart" => {
                self.emit_chart_prop(&mut attrs, props, "data", "data");
                self.emit_chart_prop(&mut attrs, props, "categories", "categories");
                self.emit_chart_prop(&mut attrs, props, "index", "index");
                self.emit_chart_prop(&mut attrs, props, "colors", "colors");
                self.emit_chart_prop(&mut attrs, props, "margin", "margin");
                self.emit_chart_prop(&mut attrs, props, "filter-opacity", "filter-opacity");
                self.emit_chart_prop(&mut attrs, props, "filter_opacity", "filter-opacity");
                self.emit_chart_prop(&mut attrs, props, "show-x-axis", "show-x-axis");
                self.emit_chart_prop(&mut attrs, props, "show_x_axis", "show-x-axis");
                self.emit_chart_prop(&mut attrs, props, "show-y-axis", "show-y-axis");
                self.emit_chart_prop(&mut attrs, props, "show_y_axis", "show-y-axis");
                self.emit_chart_prop(&mut attrs, props, "show-tooltip", "show-tooltip");
                self.emit_chart_prop(&mut attrs, props, "show_tooltip", "show-tooltip");
                self.emit_chart_prop(&mut attrs, props, "show-legend", "show-legend");
                self.emit_chart_prop(&mut attrs, props, "show_legend", "show-legend");
                self.emit_chart_prop(&mut attrs, props, "show-grid-line", "show-grid-line");
                self.emit_chart_prop(&mut attrs, props, "show_grid_line", "show-grid-line");
                self.emit_chart_prop(&mut attrs, props, "x-formatter", "x-formatter");
                self.emit_chart_prop(&mut attrs, props, "x_formatter", "x-formatter");
                self.emit_chart_prop(&mut attrs, props, "y-formatter", "y-formatter");
                self.emit_chart_prop(&mut attrs, props, "y_formatter", "y-formatter");
                self.emit_curve_type_prop(&mut attrs, props);
                self.emit_chart_prop(&mut attrs, props, "show-gradient", "show-gradient");
                self.emit_chart_prop(&mut attrs, props, "show_gradient", "show-gradient");
                if let Some(value) = props.get("custom-tooltip").or_else(|| props.get("custom_tooltip")) {
                    if let AuraPropValue::Expr(crate::ast::Expr::Ident(name)) = value {
                        attrs.push(format!(":custom-tooltip=\"{}\"", name));
                    } else if let Some(name) = self.extract_string_value(value) {
                        attrs.push(format!(":custom-tooltip=\"{}\"", name));
                    }
                }
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    if !class.is_empty() {
                        attrs.push(format!("class=\"{}\"", class));
                    }
                }
            }

            "bar_chart" | "bar-chart" => {
                self.emit_chart_prop(&mut attrs, props, "data", "data");
                self.emit_chart_prop(&mut attrs, props, "categories", "categories");
                self.emit_chart_prop(&mut attrs, props, "index", "index");
                self.emit_chart_prop(&mut attrs, props, "colors", "colors");
                self.emit_chart_prop(&mut attrs, props, "margin", "margin");
                self.emit_chart_prop(&mut attrs, props, "filter-opacity", "filter-opacity");
                self.emit_chart_prop(&mut attrs, props, "filter_opacity", "filter-opacity");
                self.emit_chart_prop(&mut attrs, props, "show-x-axis", "show-x-axis");
                self.emit_chart_prop(&mut attrs, props, "show_x_axis", "show-x-axis");
                self.emit_chart_prop(&mut attrs, props, "show-y-axis", "show-y-axis");
                self.emit_chart_prop(&mut attrs, props, "show_y_axis", "show-y-axis");
                self.emit_chart_prop(&mut attrs, props, "show-tooltip", "show-tooltip");
                self.emit_chart_prop(&mut attrs, props, "show_tooltip", "show-tooltip");
                self.emit_chart_prop(&mut attrs, props, "show-legend", "show-legend");
                self.emit_chart_prop(&mut attrs, props, "show_legend", "show-legend");
                self.emit_chart_prop(&mut attrs, props, "show-grid-line", "show-grid-line");
                self.emit_chart_prop(&mut attrs, props, "show_grid_line", "show-grid-line");
                self.emit_chart_prop(&mut attrs, props, "x-formatter", "x-formatter");
                self.emit_chart_prop(&mut attrs, props, "x_formatter", "x-formatter");
                self.emit_chart_prop(&mut attrs, props, "y-formatter", "y-formatter");
                self.emit_chart_prop(&mut attrs, props, "y_formatter", "y-formatter");
                self.emit_chart_prop(&mut attrs, props, "type", "type");
                self.emit_chart_prop(&mut attrs, props, "rounded-corners", "rounded-corners");
                self.emit_chart_prop(&mut attrs, props, "rounded_corners", "rounded-corners");
                if let Some(value) = props.get("custom-tooltip").or_else(|| props.get("custom_tooltip")) {
                    if let AuraPropValue::Expr(crate::ast::Expr::Ident(name)) = value {
                        attrs.push(format!(":custom-tooltip=\"{}\"", name));
                    } else if let Some(name) = self.extract_string_value(value) {
                        attrs.push(format!(":custom-tooltip=\"{}\"", name));
                    }
                }
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    if !class.is_empty() {
                        attrs.push(format!("class=\"{}\"", class));
                    }
                }
            }

            "line_chart" | "line-chart" => {
                self.emit_chart_prop(&mut attrs, props, "data", "data");
                self.emit_chart_prop(&mut attrs, props, "categories", "categories");
                self.emit_chart_prop(&mut attrs, props, "index", "index");
                self.emit_chart_prop(&mut attrs, props, "colors", "colors");
                self.emit_chart_prop(&mut attrs, props, "margin", "margin");
                self.emit_chart_prop(&mut attrs, props, "filter-opacity", "filter-opacity");
                self.emit_chart_prop(&mut attrs, props, "filter_opacity", "filter-opacity");
                self.emit_chart_prop(&mut attrs, props, "show-x-axis", "show-x-axis");
                self.emit_chart_prop(&mut attrs, props, "show_x_axis", "show-x-axis");
                self.emit_chart_prop(&mut attrs, props, "show-y-axis", "show-y-axis");
                self.emit_chart_prop(&mut attrs, props, "show_y_axis", "show-y-axis");
                self.emit_chart_prop(&mut attrs, props, "show-tooltip", "show-tooltip");
                self.emit_chart_prop(&mut attrs, props, "show_tooltip", "show-tooltip");
                self.emit_chart_prop(&mut attrs, props, "show-legend", "show-legend");
                self.emit_chart_prop(&mut attrs, props, "show_legend", "show-legend");
                self.emit_chart_prop(&mut attrs, props, "show-grid-line", "show-grid-line");
                self.emit_chart_prop(&mut attrs, props, "show_grid_line", "show-grid-line");
                self.emit_chart_prop(&mut attrs, props, "x-formatter", "x-formatter");
                self.emit_chart_prop(&mut attrs, props, "x_formatter", "x-formatter");
                self.emit_chart_prop(&mut attrs, props, "y-formatter", "y-formatter");
                self.emit_chart_prop(&mut attrs, props, "y_formatter", "y-formatter");
                self.emit_curve_type_prop(&mut attrs, props);
                if let Some(value) = props.get("custom-tooltip").or_else(|| props.get("custom_tooltip")) {
                    if let AuraPropValue::Expr(crate::ast::Expr::Ident(name)) = value {
                        attrs.push(format!(":custom-tooltip=\"{}\"", name));
                    } else if let Some(name) = self.extract_string_value(value) {
                        attrs.push(format!(":custom-tooltip=\"{}\"", name));
                    }
                }
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    if !class.is_empty() {
                        attrs.push(format!("class=\"{}\"", class));
                    }
                }
            }

            "donut_chart" | "donut-chart" => {
                self.emit_chart_prop(&mut attrs, props, "data", "data");
                self.emit_chart_prop(&mut attrs, props, "category", "category");
                self.emit_chart_prop(&mut attrs, props, "index", "index");
                self.emit_chart_prop(&mut attrs, props, "colors", "colors");
                self.emit_chart_prop(&mut attrs, props, "margin", "margin");
                self.emit_chart_prop(&mut attrs, props, "filter-opacity", "filter-opacity");
                self.emit_chart_prop(&mut attrs, props, "filter_opacity", "filter-opacity");
                self.emit_chart_prop(&mut attrs, props, "show-tooltip", "show-tooltip");
                self.emit_chart_prop(&mut attrs, props, "show_tooltip", "show-tooltip");
                self.emit_chart_prop(&mut attrs, props, "show-legend", "show-legend");
                self.emit_chart_prop(&mut attrs, props, "show_legend", "show-legend");
                self.emit_chart_prop(&mut attrs, props, "type", "type");
                self.emit_chart_prop(&mut attrs, props, "value-formatter", "value-formatter");
                self.emit_chart_prop(&mut attrs, props, "value_formatter", "value-formatter");
                self.emit_chart_prop(&mut attrs, props, "sort-function", "sort-function");
                self.emit_chart_prop(&mut attrs, props, "sort_function", "sort-function");
                if let Some(value) = props.get("custom-tooltip").or_else(|| props.get("custom_tooltip")) {
                    if let AuraPropValue::Expr(crate::ast::Expr::Ident(name)) = value {
                        attrs.push(format!(":custom-tooltip=\"{}\"", name));
                    } else if let Some(name) = self.extract_string_value(value) {
                        attrs.push(format!(":custom-tooltip=\"{}\"", name));
                    }
                }
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    if !class.is_empty() {
                        attrs.push(format!("class=\"{}\"", class));
                    }
                }
            }

            _ => {
                // Default handling for other components - extract class/style
                if let Some(value) = self.get_style_class(props) {
                    let class = self.extract_string_value(value).unwrap_or("");
                    if !class.is_empty() {
                        attrs.push(format!("class=\"{}\"", class));
                    }
                }
            }
        }

        // Add event handlers
        for (event, aura_event) in events {
            // .window/.document modifiers → global listener, no template attr
            if self.try_register_global_listener(event, aura_event) {
                continue;
            }
            let vue_event = self.shadcn_event_to_vue(tag, event);
            let mut handler_fn = self.handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
            // Track used handler (without params for matching)
            let handler_name = self.handler_to_function_call(&aura_event.handler);
            // If inside a for-loop, pass the loop variable's .id as argument (e.g., SelectNote(note.id))
            // Only append if handler doesn't already have params from aura_event
            if let Some(ref loop_var) = self.current_loop_var {
                if aura_event.params.is_empty() {
                    handler_fn = format!("{}({})", handler_fn, loop_var);
                    self.loop_param_handlers.insert(handler_name.clone(), loop_var.clone());
                }
            }
            self.used_handlers.insert(handler_name);
            attrs.push(format!("{}=\"{}\"", vue_event, handler_fn));
        }

        (attrs, slot_content, slot_children)
    }

    /// Convert AutoUI event to Vue event for shadcn-vue components
    fn shadcn_event_to_vue(&self, _tag: &str, event: &str) -> String {
        let (base, modifiers) = Self::split_event_key(event);
        let mut vue = match base {
            "onclick" | "onClick" | "on_click" => "@click".to_string(),
            "oninput" | "onInput" => "@update:modelValue".to_string(),
            "onchange" | "onChange" => "@update:modelValue".to_string(),
            "onenter" | "onEnter" => "@keyup.enter".to_string(),
            _ => format!("@{}", Self::base_event_to_dom(base)),
        };
        for m in modifiers {
            if m == "window" || m == "document" {
                continue; // global target — handled elsewhere
            }
            vue.push('.');
            vue.push_str(Self::vue_modifier(m));
        }
        vue
    }

    /// Extract string value from AuraPropValue
    fn extract_string_value<'a>(&self, value: &'a AuraPropValue) -> Option<&'a str> {
        match value {
            AuraPropValue::Expr(crate::ast::Expr::Str(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Extract boolean value from AuraPropValue
    fn extract_bool_value(&self, value: &AuraPropValue) -> bool {
        match value {
            AuraPropValue::Expr(crate::ast::Expr::Bool(b)) => *b,
            AuraPropValue::Expr(crate::ast::Expr::Str(s)) => s == "true",
            _ => false,
        }
    }

    /// Extract integer value from AuraPropValue
    fn extract_int_value(&self, value: &AuraPropValue) -> Option<i64> {
        match value {
            AuraPropValue::Expr(crate::ast::Expr::Int(n)) => Some(*n as i64),
            AuraPropValue::Expr(crate::ast::Expr::Str(s)) => s.parse().ok(),
            _ => None,
        }
    }

    /// Extract float value from AuraPropValue
    fn extract_float_value(&self, value: &AuraPropValue) -> Option<f64> {
        match value {
            AuraPropValue::Expr(crate::ast::Expr::Float(n, _)) => Some(*n),
            AuraPropValue::Expr(crate::ast::Expr::Int(n)) => Some(*n as f64),
            AuraPropValue::Expr(crate::ast::Expr::Str(s)) => s.parse().ok(),
            _ => None,
        }
    }

    /// Extract state reference from AuraPropValue
    ///
    /// Accepts every shape the parser can produce for a `.state` ref:
    /// - `Ident("name")` — bare identifier (hand-built ASTs, some legacy paths)
    /// - `Ident(".name")` — legacy dot-prefixed identifier (dot stripped)
    /// - `Dot(Ident("self") | Ident("."), "name")` — what `dot_item` actually
    ///   produces for `.name` in real widget source (parser.rs dot_item).
    fn extract_state_ref(&self, value: &AuraPropValue) -> Option<String> {
        match value {
            AuraPropValue::Expr(crate::ast::Expr::Ident(name)) => {
                let stripped = name.as_str().strip_prefix('.').unwrap_or(name.as_str());
                if stripped.is_empty() {
                    None
                } else {
                    Some(stripped.to_string())
                }
            }
            AuraPropValue::Expr(crate::ast::Expr::Dot(obj, field)) => match obj.as_ref() {
                crate::ast::Expr::Ident(obj_name)
                    if obj_name.as_str() == "self" || obj_name.as_str() == "." =>
                {
                    Some(field.to_string())
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// Get style or class prop value (style takes priority over class)
    /// This supports the transition from 'class' to 'style' prop naming
    fn get_style_class<'a>(&self, props: &'a HashMap<String, AuraPropValue>) -> Option<&'a AuraPropValue> {
        props.get("style").or_else(|| props.get("class"))
    }

    /// Build a `:kebab-attr="value"` binding for a boolean-style prop that
    /// also accepts an expression reference (e.g. `can_edit: .editable`).
    ///
    /// Looks up the prop under both `snake_key` (AURA convention, e.g.
    /// `can_edit`) and `camel_key` (Vue convention, e.g. `canEdit`). When the
    /// prop is present as a bool/bool-string literal, emits the literal value.
    /// When present as another expression, emits the bound JS value. When
    /// absent, emits `default_val` so the consumer always gets a defined prop.
    /// (Plan 354 Phase C: used by AutoDownEditor's can_edit / show_actions.)
    fn bool_prop_binding(
        &self,
        props: &HashMap<String, AuraPropValue>,
        snake_key: &str,
        camel_key: &str,
        default_val: bool,
    ) -> String {
        // Derive the kebab-case Vue attribute from the camelCase key
        // (canEdit → can-edit, showActions → show-actions).
        let kebab_attr: String = camel_key.chars().fold(String::new(), |acc, c| {
            if c.is_ascii_uppercase() {
                if acc.is_empty() {
                    c.to_ascii_lowercase().to_string()
                } else {
                    format!("{}-{}", acc, c.to_ascii_lowercase())
                }
            } else {
                format!("{}{}", acc, c)
            }
        });
        match props.get(snake_key).or_else(|| props.get(camel_key)) {
            Some(AuraPropValue::Expr(crate::ast::Expr::Bool(b))) => {
                format!(":{}=\"{}\"", kebab_attr, b)
            }
            Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) => {
                format!(":{}=\"{}\"", kebab_attr, s == "true")
            }
            Some(AuraPropValue::Expr(expr)) => {
                // Dynamic expression (e.g. can_edit: .editable → editable).
                match self.expr_to_vue_bound_value(expr) {
                    Ok(js_expr) => format!(":{}=\"{}\"", kebab_attr, js_expr),
                    Err(_) => format!(":{}=\"{}\"", kebab_attr, default_val),
                }
            }
            _ => format!(":{}=\"{}\"", kebab_attr, default_val),
        }
    }

    /// Split an event key into (base, modifiers).
    /// "onwheel.document.capture" → ("onwheel", ["document", "capture"])
    ///
    /// Quoted custom event names (`on"autodown:slash-open"`) may contain
    /// characters that are illegal in identifiers (':'/'-'); the base then
    /// runs to the closing quote and only what follows it is modifiers.
    fn split_event_key(event: &str) -> (&str, Vec<&str>) {
        if let Some(rest) = event.strip_prefix("on\"") {
            if let Some(close) = rest.find('"') {
                let end = 3 + close + 1; // just past the closing quote
                let base = &event[..end];
                let mods: Vec<&str> = event[end..]
                    .strip_prefix('.')
                    .map(|t| t.split('.').filter(|m| !m.is_empty()).collect())
                    .unwrap_or_default();
                return (base, mods);
            }
        }
        let mut parts = event.split('.');
        let base = parts.next().unwrap_or(event);
        (base, parts.collect())
    }

    /// Map the base event key (without modifiers) to the DOM/Vue event name.
    /// Covers the full common set: keyboard, mouse, wheel, focus, pointer,
    /// touch, drag, scroll. Unknown `onxxx` keys fall back to stripping `on`.
    /// Quoted custom names (`on"autodown:slash-open"`) map to the raw name.
    fn base_event_to_dom(base: &str) -> String {
        if let Some(inner) = base.strip_prefix("on\"").and_then(|b| b.strip_suffix('"')) {
            return inner.to_string();
        }
        match base.to_ascii_lowercase().as_str() {
            "onclick" | "on_click" => "click",
            "ondblclick" | "on_double_click" => "dblclick",
            "oninput" => "input",
            "onchange" => "change",
            "onblur" => "blur",
            "onfocus" => "focus",
            "onfocusin" => "focusin",
            "onfocusout" => "focusout",
            // Keyboard
            "onkeydown" => "keydown",
            "onkeyup" => "keyup",
            "onkeypress" => "keypress",
            // Mouse
            "onmousedown" => "mousedown",
            "onmouseup" => "mouseup",
            "onmousemove" => "mousemove",
            "onmouseenter" => "mouseenter",
            "onmouseleave" => "mouseleave",
            "onmouseover" => "mouseover",
            "onmouseout" => "mouseout",
            // Wheel / context menu / scroll
            "onwheel" => "wheel",
            "oncontextmenu" => "contextmenu",
            "onscroll" => "scroll",
            // Pointer Events
            "onpointerdown" => "pointerdown",
            "onpointerup" => "pointerup",
            "onpointermove" => "pointermove",
            "onpointerenter" => "pointerenter",
            "onpointerleave" => "pointerleave",
            "onpointerover" => "pointerover",
            "onpointerout" => "pointerout",
            "onpointercancel" => "pointercancel",
            // Touch
            "ontouchstart" => "touchstart",
            "ontouchmove" => "touchmove",
            "ontouchend" => "touchend",
            "ontouchcancel" => "touchcancel",
            // Drag & drop
            "ondragstart" => "dragstart",
            "ondrag" => "drag",
            "ondragend" => "dragend",
            "ondragover" => "dragover",
            "ondragenter" => "dragenter",
            "ondragleave" => "dragleave",
            "ondrop" => "drop",
            // Fallback: strip a single leading "on" (onupdate → update, etc.)
            other => other.strip_prefix("on").unwrap_or(other),
        }
        .to_string()
    }

    /// Normalize an event modifier for Vue templates.
    /// Auto names are mapped to Vue's modifier vocabulary where they differ.
    fn vue_modifier(m: &str) -> &str {
        match m {
            "escape" => "esc",
            "del" => "delete",
            other => other,
        }
    }

    /// Convert AutoUI event name (with optional `.modifier` chain) to a Vue
    /// template event binding, e.g. `onkeydown.up.prevent` → `@keydown.up.prevent`.
    ///
    /// `.window` / `.document` modifiers are NOT handled here — they mark
    /// global listeners and are intercepted by `try_register_global_listener`
    /// before this function is called.
    fn auto_event_to_vue(&self, event: &str) -> String {
        let (base, modifiers) = Self::split_event_key(event);
        // Existing shorthands keep their historical expansion.
        let mut vue = match base {
            "onenter" | "onEnter" => "@keyup.enter".to_string(),
            "onsubmit" | "onSubmit" => "@submit.prevent".to_string(),
            _ => format!("@{}", Self::base_event_to_dom(base)),
        };
        for m in modifiers {
            if m == "window" || m == "document" {
                continue; // global target — handled elsewhere
            }
            vue.push('.');
            vue.push_str(Self::vue_modifier(m));
        }
        vue
    }

    /// Render the addEventListener/removeEventListener options argument
    /// (", { capture: true, passive: false }" or ""). For removal only the
    /// capture flag matters (per DOM spec), so passive is omitted there.
    fn listener_options(gl: &GlobalListener, for_removal: bool) -> String {
        let mut opts: Vec<String> = Vec::new();
        if gl.capture {
            opts.push("capture: true".to_string());
        }
        if !for_removal {
            if let Some(passive) = gl.passive {
                opts.push(format!("passive: {}", passive));
            }
        }
        if opts.is_empty() {
            String::new()
        } else {
            format!(", {{ {} }}", opts.join(", "))
        }
    }

    /// Make an arbitrary DOM event name safe for embedding in a JS identifier.
    /// Custom event names may contain ':'/'-' (e.g. "autodown:slash-open"),
    /// which would otherwise produce an invalid wrapper function name.
    fn sanitize_ident(s: &str) -> String {
        s.chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
            .collect()
    }

    /// Intercept `.window` / `.document` event modifiers and register a global
    /// listener instead of emitting a template attribute. Returns true when
    /// the event was claimed (caller must skip normal attribute emission).
    ///
    /// Declaration: `onmousemove.window: .DragMove($event)` or
    /// `onwheel.document.capture.prevent: .LockWheel($event)`.
    fn try_register_global_listener(&mut self, event: &str, aura_event: &AuraEvent) -> bool {
        let (base, modifiers) = Self::split_event_key(event);
        let target = modifiers
            .iter()
            .find(|m| **m == "window" || **m == "document");
        let target = match target {
            Some(t) => t.to_string(),
            None => return false,
        };

        let dom_event = Self::base_event_to_dom(base);
        let prevent = modifiers.iter().any(|m| *m == "prevent");
        let stop = modifiers.iter().any(|m| *m == "stop");
        let capture = modifiers.iter().any(|m| *m == "capture");
        let passive = if modifiers.iter().any(|m| *m == "passive") {
            Some(true)
        } else if prevent {
            // Chrome treats document/window wheel & touch listeners as passive
            // by default — preventDefault requires an explicit passive: false.
            Some(false)
        } else {
            None
        };

        let handler_fn = self.handler_to_function_call(&aura_event.handler);
        self.used_handlers.insert(handler_fn.clone());

        // Build the handler call. In addEventListener context the DOM event is
        // the listener's argument `e` (Vue's `$event` does not exist here).
        let call_args: Vec<String> = if aura_event.params.is_empty() {
            vec!["e".to_string()]
        } else {
            aura_event.params
                .iter()
                .map(|p| {
                    let stripped = p.strip_prefix("this.").unwrap_or(p);
                    stripped.replace("$event", "e").replace('"', "'")
                })
                .collect()
        };
        let call = format!("{}({})", handler_fn, call_args.join(", "));

        // A wrapper is needed when prevent/stop must run or when the handler
        // takes adapted arguments. Otherwise the bare function ref suffices
        // (the DOM passes the event object as first argument).
        let needs_wrapper = prevent || stop || !aura_event.params.is_empty();
        let (listener, wrapper) = if needs_wrapper {
            let wrapper_fn = format!("__auto_gl_{}_{}", Self::sanitize_ident(&dom_event), handler_fn);
            let mut body = String::new();
            if stop {
                body.push_str("  e.stopPropagation()\n");
            }
            if prevent {
                body.push_str("  e.preventDefault()\n");
            }
            body.push_str(&format!("  {}\n", call));
            // `e: any` — the concrete type (KeyboardEvent/MouseEvent/WheelEvent)
            // depends on the DOM event; `any` keeps field access (e.clientY,
            // e.deltaY) type-checkable without narrowing machinery.
            let src = format!("function {}(e: any) {{\n{}}}\n\n", wrapper_fn, body);
            (wrapper_fn, Some(src))
        } else {
            (handler_fn, None)
        };

        let entry = GlobalListener {
            target,
            event: dom_event,
            listener,
            capture,
            passive,
            wrapper,
        };
        // Dedup identical registrations (same listener may be declared on
        // multiple elements).
        if !self.global_listeners.iter().any(|g| {
            g.target == entry.target && g.event == entry.event && g.listener == entry.listener
        }) {
            self.global_listeners.push(entry);
        }
        true
    }

    /// Plan 367 P0-1: Convert PascalCase to snake_case for callback prop matching.
    /// e.g. "TogglePin" → "toggle_pin", "DeleteActive" → "delete_active"
    fn pascal_to_snake(s: &str) -> String {
        let mut result = String::new();
        for (i, ch) in s.chars().enumerate() {
            if ch.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }
        result
    }

    /// Plan 367 P0-2: Indent every line of a body string by the given prefix.
    /// Used when wrapping transpiled handler bodies into function/onMounted.
    /// Ensures multi-line bodies have consistent indentation.
    fn indent_body(body: &str, indent: &str) -> String {
        body.lines()
            .map(|line| {
                if line.trim().is_empty() {
                    String::new()
                } else {
                    format!("{}{}", indent, line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Plan 367 P0-4/P1-1: Map an Auto type to its TypeScript equivalent.
    /// Used for emit payload types and prop types.
    fn auto_type_to_ts_type(ty: &crate::ast::Type) -> String {
        use crate::ast::Type;
        match ty {
            Type::StrSlice | Type::StrOwned | Type::StrFixed(_) | Type::CStrLit => "string".to_string(),
            Type::Int | Type::I64 | Type::Uint | Type::U64 | Type::USize | Type::Float | Type::Double => "number".to_string(),
            Type::Bool => "boolean".to_string(),
            Type::List(inner) => format!("{}[]", Self::auto_type_to_ts_type(inner)),
            Type::Slice(_) => "any[]".to_string(),
            Type::Option(inner) => format!("{} | null", Self::auto_type_to_ts_type(inner)),
            Type::User(decl) => {
                let name = decl.name.as_str();
                match name {
                    "msg" => "() => void".to_string(),
                    "str" => "string".to_string(),
                    "int" | "i64" | "uint" => "number".to_string(),
                    "bool" => "boolean".to_string(),
                    // Custom types (e.g. Note) — use the type name directly.
                    // The interface should be imported from api.ts.
                    other => other.to_string(),
                }
            }
            _ => "any".to_string(),
        }
    }

    /// Strip a trailing param list from an on-block handler pattern key.
    /// Plan 374 embeds param names in handler keys (".Scrolled(e)") so the
    /// Rust backend can recover them; the Vue backend matches handlers by
    /// base name (".Scrolled"), so normalize before lookup.
    fn base_pattern(pattern: &str) -> &str {
        match pattern.find('(') {
            Some(i) if pattern.ends_with(')') => &pattern[..i],
            _ => pattern,
        }
    }

    /// Look up on-block handler params by base pattern (".Name"), tolerant of
    /// Plan 374 parameterized keys (".Name(e)").
    fn get_handler_params<'a>(
        handler_params: &'a std::collections::HashMap<String, Vec<String>>,
        base_key: &str,
    ) -> Option<&'a Vec<String>> {
        handler_params
            .iter()
            .find(|(k, _)| Self::base_pattern(k) == base_key)
            .map(|(_, v)| v)
    }

    /// Convert handler pattern to function name
    ///
    /// The result must be a valid JS identifier: emit names may contain
    /// characters like ':' ("update:modelValue" v-model contracts), so the
    /// final name is sanitized (update:modelValue → update_modelValue). The
    /// verbatim emit name is recovered via the on-block pattern when needed.
    fn pattern_to_handler_name(&self, pattern: &str) -> String {
        let pattern = Self::base_pattern(pattern);
        let name = if pattern.starts_with('.') {
            // Dot-prefixed handlers map directly to function name (Vue convention)
            pattern[1..].to_string()
        } else if let Some(variant) = pattern.split("::").last() {
            // Pattern like "Msg::Inc" -> "onInc"
            format!("on{}", variant)
        } else {
            format!("on{}", pattern)
        };
        Self::sanitize_ident(&name)
    }

    /// Convert handler reference to function call
    /// (same sanitization as pattern_to_handler_name — the two must agree so
    /// template references match the generated function definitions)
    fn handler_to_function_call(&self, handler: &str) -> String {
        let handler = Self::base_pattern(handler);
        let name = if handler.starts_with('.') {
            // Dot-prefixed handlers map directly to function name (Vue convention)
            handler[1..].to_string()
        } else if let Some(variant) = handler.split("::").last() {
            // Handler like "Msg::Inc" -> "onInc"
            format!("on{}", variant)
        } else {
            format!("on{}", handler)
        };
        Self::sanitize_ident(&name)
    }

    /// Find the state variable that likely represents the "active id" for a given handler.
    /// Heuristic: looks for state vars ending with "_id" (e.g., "active_id" for "SelectNote")
    fn find_active_id_var(&self, _handler_name: &str) -> Option<String> {
        // Look for a state variable ending with "_id"
        for name in &self.state_names {
            if name.ends_with("_id") {
                return Some(name.clone());
            }
        }
        None
    }

    /// Convert handler to Vue function call with parameters
    fn handler_to_function_call_with_params(&self, handler: &str, params: &[String]) -> String {
        let func_name = self.handler_to_function_call(handler);
        if params.is_empty() {
            func_name
        } else {
            // Replace double quotes with single quotes in params to avoid HTML attr quoting issues
            let safe_params: Vec<String> = params.iter()
                .map(|p| {
                    // Plan 345: event-arg parser emits standalone `.field` as
                    // `this.field` (correct for ArkTS). Vue <script setup> uses
                    // bare state refs, so strip a leading `this.` here.
                    let stripped = p.strip_prefix("this.").unwrap_or(p);
                    stripped.replace('"', "'")
                })
                .collect();
            format!("{}({})", func_name, safe_params.join(", "))
        }
    }

    // ========================================================================
    // shadcn-vue Support Methods (using unified WidgetRegistry)
    // ========================================================================

    /// Register a shadcn-vue component as used
    fn register_shadcn_component(&mut self, tag: &str) {
        if self.is_shadcn() {
            if let Some(component_name) = self.widget_registry.get_primary_component("vue", tag) {
                self.shadcn_components_used.insert(component_name);
            }
        }
    }

    /// Generate shadcn-vue import statements using unified registry
    fn generate_shadcn_imports(&self) -> String {
        if self.shadcn_components_used.is_empty() {
            return String::new();
        }

        // Collect all tags used and their imports
        let mut imports_by_path: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        for component_name in &self.shadcn_components_used {
            // Find the widget spec that contains this component
            for (_, spec) in self.widget_registry.all_widgets().iter() {
                if let Some(mapping) = spec.backend("vue") {
                    if &mapping.component == component_name || mapping.extra_components.contains(component_name) {
                        if let Some(ref import_path) = mapping.import {
                            imports_by_path.entry(import_path.clone()).or_default().push(component_name.clone());
                        }
                    }
                }
            }
        }

        // Generate import statements
        let mut imports = Vec::new();
        for (path, mut names) in imports_by_path {
            names.sort();
            names.dedup();
            imports.push(format!("import {{ {} }} from '{}'\n", names.join(", "), path));
        }

        imports.sort();
        imports.join("")
    }

    /// Get shadcn-vue component name for a tag
    fn shadcn_component_name(&self, tag: &str) -> Option<String> {
        if self.is_shadcn() {
            self.widget_registry.get_primary_component("vue", tag)
        } else {
            None
        }
    }

    /// Generate components.json for shadcn-vue project setup
    pub fn generate_components_json() -> String {
        r#"{
  "$schema": "https://shadcn-vue.com/schema.json",
  "style": "default",
  "typescript": true,
  "tailwind": {
    "config": "tailwind.config.cjs",
    "css": "src/assets/index.css",
    "baseColor": "slate",
    "cssVariables": true
  },
  "aliases": {
    "components": "@/components",
    "utils": "@/lib/utils"
  }
}"#.to_string()
    }

    /// Generate package.json for shadcn-vue project
    pub fn generate_package_json(project_name: &str) -> String {
        format!(r#"{{
  "name": "{}",
  "version": "0.0.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "vue": "^3.4.0",
    "vue-router": "^4.2.0",
    "@vueuse/core": "^10.7.0",
    "radix-vue": "^1.4.0",
    "reka-ui": "^2.0.0",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.2.0",
    "lucide-vue-next": "^0.312.0",
    "embla-carousel-vue": "^8.5.1",
    "vee-validate": "^4.15.1",
    "@vee-validate/zod": "^4.15.1",
    "zod": "^3.25.76",
    "prismjs": "^1.29.0"
  }},
  "devDependencies": {{
    "@vitejs/plugin-vue": "^5.0.0",
    "vite": "^5.0.0",
    "typescript": "^5.3.0",
    "vue-tsc": "^2.0.0",
    "tailwindcss": "^3.4.0",
    "tailwindcss-animate": "^1.0.7",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0",
    "@types/prismjs": "^1.26.0"
  }}
}}"#, project_name)
    }

    /// Generate vite.config.ts for shadcn-vue project
    pub fn generate_vite_config() -> String {
        r#"import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  build: {
    rollupOptions: {
      output: {
        entryFileNames: 'assets/index.js',
        chunkFileNames: 'assets/[name].js',
        assetFileNames: 'assets/[name].[ext]',
      },
    },
  },
})
"#.to_string()
    }

    /// Generate tailwind.config.js for shadcn-vue project
    pub fn generate_tailwind_config() -> String {
        r#"/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["class"],
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
      },
	      borderRadius: {
	        lg: "var(--radius)",
	        md: "calc(var(--radius) - 2px)",
	        sm: "calc(var(--radius) - 4px)",
	      },
	      keyframes: {
	        "accordion-down": {
	          from: { height: 0 },
	          to: { height: "var(--radix-accordion-content-height)" },
	        },
	        "accordion-up": {
	          from: { height: "var(--radix-accordion-content-height)" },
	          to: { height: 0 },
	        },
	        "collapsible-down": {
	          from: { height: 0, opacity: 0 },
	          to: { height: "var(--radix-collapsible-content-height)", opacity: 1 },
	        },
	        "collapsible-up": {
	          from: { height: "var(--radix-collapsible-content-height)", opacity: 1 },
	          to: { height: 0, opacity: 0 },
	        },
	      },
	      animation: {
	        "accordion-down": "accordion-down 0.2s ease-out",
	        "accordion-up": "accordion-up 0.2s ease-out",
	        "collapsible-down": "collapsible-down 0.2s ease-out",
	        "collapsible-up": "collapsible-up 0.2s ease-out",
	      },
	      boxShadow: {
	        "card": "var(--card-shadow)",
	      },
    },
  },
  plugins: [require("tailwindcss-animate")],
}
"#.to_string()
    }

    /// Generate lib/utils.ts for shadcn-vue project
    pub fn generate_utils_ts() -> String {
        r#"import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
"#.to_string()
    }

	    /// Generate base CSS file with CSS variables
	    pub fn generate_base_css() -> String {
	        r#"@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 222.2 84% 4.9%;
    --card: 0 0% 100%;
    --card-foreground: 222.2 84% 4.9%;
    --popover: 0 0% 100%;
    --popover-foreground: 222.2 84% 4.9%;
    --primary: 222.2 47.4% 11.2%;
    --primary-foreground: 210 40% 98%;
    --secondary: 210 40% 96.1%;
    --secondary-foreground: 222.2 47.4% 11.2%;
    --muted: 210 40% 96.1%;
    --muted-foreground: 215.4 16.3% 46.9%;
    --accent: 210 40% 96.1%;
    --accent-foreground: 222.2 47.4% 11.2%;
    --destructive: 0 84.2% 60.2%;
    --destructive-foreground: 210 40% 98%;
    --border: 214.3 31.8% 91.4%;
    --input: 214.3 31.8% 91.4%;
    --ring: 222.2 84% 4.9%;
    --radius: 0.5rem;

    /* Plan 360: card shadows — deeper in dark mode */
    --card-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1);
  }

  .dark {
    --background: 222.2 47% 7%;
    --foreground: 210 40% 98%;
    --card: 222.2 47% 9%;
    --card-foreground: 210 40% 98%;
    --popover: 222.2 47% 9%;
    --popover-foreground: 210 40% 98%;
    --primary: 210 40% 98%;
    --primary-foreground: 222.2 47.4% 11.2%;
    --secondary: 217.2 32.6% 17.5%;
    --secondary-foreground: 210 40% 98%;
    --muted: 217.2 32.6% 17.5%;
    --muted-foreground: 215 20.2% 65.1%;
    --accent: 217.2 32.6% 17.5%;
    --accent-foreground: 210 40% 98%;
    --destructive: 0 62.8% 30.6%;
    --destructive-foreground: 210 40% 98%;
    --border: 217.2 32.6% 17.5%;
    --input: 217.2 32.6% 17.5%;
    --ring: 212.7 26.8% 83.9%;

    /* Plan 360: deeper shadows in dark mode for visual depth */
    --card-shadow: 0 4px 12px 0 rgb(0 0 0 / 0.4), 0 2px 4px -2px rgb(0 0 0 / 0.3);
  }
}

/* Plan 360: custom card shadow utility */
@layer utilities {
  .shadow-card {
    box-shadow: var(--card-shadow);
  }
}

@layer base {
  * {
    @apply border-border;
  }
  body {
    @apply bg-background text-foreground;
    /* Plan 360: smooth dark mode transitions */
    transition: background-color 0.3s ease, color 0.3s ease;
  }
}
"#.to_string()
    }

    /// Generate a composable singleton `.ts` file for a shared store
    /// (Plan 351 / Design 18). Produces module-level `ref`s + an exported
    /// `useXxxStore()` function returning state refs and action functions.
    pub fn generate_store_composable(store: &crate::aura::AuraStore) -> String {
        use crate::ui_gen::ts_adapter::{transpile_handler_body, AuraTsContext};

        let mut code = String::new();
        code.push_str("import { ref } from 'vue'\n");
        // API functions used in handler bodies (from `use back.api`)
        if !store.api_imports.is_empty() {
            let fns = store.api_imports.join(", ");
            code.push_str(&format!("import {{ {} }} from '@/lib/api'\n", fns));
        }
        code.push('\n');

        // Module-level ref declarations (singleton state).
        for sv in &store.state_vars {
            let init = Self::store_init_to_js(&sv.initial);
            code.push_str(&format!("const {} = ref<any>({})\n", sv.name, init));
        }
        code.push('\n');

        // Build ctx for handler transpilation (state_names → .value emission).
        let state_names: std::collections::HashSet<String> =
            store.state_vars.iter().map(|s| s.name.clone()).collect();
        // Pass API imports so ts_adapter adds `await` to API calls.
        let ctx = AuraTsContext::new(state_names)
            .with_props(std::collections::HashSet::new())
            .with_api_functions(store.api_imports.clone());

        // Plan 360: detect accent_color state so we can inject the palette
        // helpers and expose `accent_names` as a computed getter.
        let has_accent = store.state_vars.iter().any(|s| s.name == "accent_color");

        // Export function.
        let fn_name = format!("use{}Store", store.name);
        code.push_str(&format!("export function {}(): any {{\n", fn_name));
        code.push_str("    return {\n");

        // Expose state refs by name.
        for sv in &store.state_vars {
            code.push_str(&format!("        {},\n", sv.name));
        }

        // Expose handlers as action functions.
        for (pattern, payload) in &store.handlers {
            // Plan 374: the aura extractor embeds parameter names in the pattern
            // key (e.g. ".SelectFolder(folder)"), so we must strip both the
            // leading dot AND the parameter list to recover the bare action name.
            // Otherwise we'd emit `SelectFolder(folder): (folder: any) => ...`,
            // a JS method-shorthand followed by an arrow — a syntax error.
            // Params still come from handler_params below for the arrow signature.
            let after_dot = pattern.trim_start_matches('.');
            let action_name = match after_dot.find('(') {
                Some(paren) => after_dot[..paren].to_string(),
                None => after_dot.to_string(),
            };
            let mut body = match payload {
                crate::aura::LogicPayload::AstStmts(stmts) => transpile_handler_body(stmts, &ctx),
                _ => String::new(),
            };
            // Plan 360: for accent-enabled stores, wire up CSS-variable updates
            // alongside the .at-side state changes. We append JS calls that the
            // .at source can't express directly.
            if has_accent {
                if action_name == "SetAccent" {
                    // applyAccent(name, isDark) writes the --primary CSS variable.
                    body.push_str("; applyAccent(name, dark_mode.value)");
                } else if action_name == "ToggleDarkMode" {
                    // After flipping dark_mode, re-apply accent so lightness adjusts.
                    body.push_str("; applyAccent(accent_color.value, dark_mode.value)");
                }
            }
            // Actions with API calls need to be async (ts_adapter added `await`).
            let is_async = body.contains("await");
            let params = store.handler_params.get(pattern)
                .map(|p| p.iter().map(|n| format!("{}: any", n)).collect::<Vec<_>>().join(", "))
                .unwrap_or_default();
            let async_kw = if is_async { "async " } else { "" };
            code.push_str(&format!(
                "        {}: {}({}) => {{ {} }},\n",
                action_name, async_kw, params, body
            ));
        }

        // Auto-generate a computed 'all_tags' property that collects unique tags
        // from all notes — but only if the store doesn't already declare all_tags.
        // Check both model state_vars AND the user-declared computed block: a
        // `computed { all_tags => ... }` declaration must suppress this auto-inject,
        // otherwise we'd emit a duplicate `get all_tags()` key (vite warning).
        let has_all_tags = store.state_vars.iter().any(|s| s.name == "all_tags")
            || store.computed.iter().any(|c| c.name == "all_tags");
        if !has_all_tags {
            code.push_str("        get all_tags() {\n");
            code.push_str("            const tags = new Set<string>();\n");
            code.push_str("            for (const n of notes.value) {\n");
            code.push_str("                if (n && n.tags) for (const t of n.tags) tags.add(t);\n");
            code.push_str("            }\n");
            code.push_str("            return Array.from(tags);\n");
            code.push_str("        },\n");
        }

        // Plan 360: when accent_color exists, expose the palette name list as
        // a computed so the UI can render swatch buttons via `for n in .store.accent_names`.
        if has_accent {
            code.push_str("        get accent_names() {\n");
            code.push_str("            return getAccentNames();\n");
            code.push_str("        },\n");
        }

        // Plan 367 P2-2: user-declared computed properties → JS getters.
        // Uses ts_adapter to transpile the expression, with store state_names
        // as context (so .value is appended correctly to module-level refs).
        for computed_prop in &store.computed {
            let mut buf = Vec::new();
            crate::ui_gen::ts_adapter::transpile_expr_pub(&computed_prop.expr, &ctx, &mut buf);
            let expr_js = String::from_utf8_lossy(&buf);
            code.push_str(&format!("        get {}() {{\n", computed_prop.name));
            code.push_str(&format!("            return {};\n", expr_js.trim()));
            code.push_str("        },\n");
        }

        code.push_str("    }\n");
        code.push_str("}\n");

        // Plan 360: accent color system. When the store declares an
        // `accent_color` state var, inject the 5-color palette + applyAccent
        // function + onMounted bootstrap. The palette is aligned with
        // auto-forge (indigo/coral/ocean/sage/amber).
        if has_accent {
            code.push_str("\n");
            code.push_str(Self::ACCENT_PALETTE_JS);
            // Module-level bootstrap: apply saved accent on first import.
            // Also sync the accent_color ref so the store reflects the
            // persisted choice (localStorage may differ from the .at default).
            code.push_str("\n// Restore saved accent on module load.\n");
            code.push_str(
                "(function bootstrapAccent() {\n\
                 \x20 const saved = getSavedAccent()\n\
                 \x20 accent_color.value = saved\n\
                 \x20 const isDark = document.documentElement.classList.contains('dark')\n\
                 \x20 applyAccent(saved, isDark)\n\
                 })()\n",
            );
        }

        code
    }

    /// Plan 360: JS code injected into the store composable when `accent_color`
    /// state is declared. Defines the 5-color palette, an apply function that
    /// writes the HSL triplet to the `--primary` CSS variable (and persists to
    /// localStorage), and an onMounted bootstrap that restores the saved choice.
    ///
    /// Palette values are aligned with auto-forge's useAccentColor.ts so the
    /// two products share the same visual language.
    const ACCENT_PALETTE_JS: &str = r#"
// Plan 360: Accent color palette (aligned with auto-forge).
// Each entry maps a name → shadcn --primary HSL triplet (space-separated).
const ACCENT_PALETTES: Record<string, string> = {
  indigo: '239 84% 67%',
  coral:  '350 75% 64%',
  ocean:  '217 91% 60%',
  sage:   '160 84% 39%',
  amber:  '38 92% 50%',
}
const ACCENT_NAMES = Object.keys(ACCENT_PALETTES)
const ACCENT_STORAGE_KEY = 'notes-accent-color'

/** Apply the named accent by writing the --primary CSS variable.
 *  Also adjusts lightness up slightly in dark mode for readability.
 *  HSL values are stored as "H S% L%" (shadcn format); the % is preserved
 *  so we use parseFloat to read the numeric part for the lightness tweak.
 *
 *  The variable is written to BOTH <html> AND any element carrying the
 *  `.dark` class. This is necessary because the generated dark-mode CSS
 *  puts `.dark { --primary: ... }` on a root wrapper div, which would
 *  otherwise shadow the value inherited from <html>. We use a microtask
 *  (setTimeout 0) for the .dark pass so Vue has flushed the :class change. */
export function applyAccent(name: string, isDark = false): void {
  const hsl = ACCENT_PALETTES[name]
  if (!hsl) return
  let finalHsl = hsl
  // Dark mode: boost lightness ~4% for contrast against dark backgrounds.
  if (isDark) {
    const match = hsl.match(/^(\d+\s+[\d.]+%)\s+([\d.]+)%$/)
    if (match) {
      const boosted = Math.min(85, parseFloat(match[2]) + 4)
      finalHsl = match[1] + ' ' + boosted + '%'
    }
  }
  const root = document.documentElement
  root.style.setProperty('--primary', finalHsl)
  // Also set on any .dark element so it overrides the .dark { --primary }
  // rule defined in index.css (which lives on a different element than <html>).
  // Done synchronously AND on next tick (covers both: dark already applied,
  // and dark just toggled — Vue flushes :class after this call returns).
  // CRITICAL: in light mode (no .dark elements) we must also REMOVE any
  // stale inline --primary left over from a previous dark-mode apply, otherwise
  // the old value shadows the new <html>-level value via CSS inheritance.
  function applyToDark() {
    if (isDark) {
      document.querySelectorAll('.dark').forEach(function (el) {
        ;(el as HTMLElement).style.setProperty('--primary', finalHsl)
      })
    } else {
      // Light mode: clear any stale inline --primary on elements that
      // previously carried .dark (the wrapper still exists, just without .dark).
      // IMPORTANT: skip documentElement (html) — that's where we just set the
      // current value. Only clear child elements with a stale inline override.
      document.querySelectorAll('[style*="--primary"]').forEach(function (el) {
        if (el !== document.documentElement) {
          ;(el as HTMLElement).style.removeProperty('--primary')
        }
      })
    }
  }
  applyToDark()
  setTimeout(applyToDark, 0)
  try { localStorage.setItem(ACCENT_STORAGE_KEY, name) } catch {}
}

/** Read the saved accent from localStorage, defaulting to 'indigo'. */
export function getSavedAccent(): string {
  try {
    const saved = localStorage.getItem(ACCENT_STORAGE_KEY)
    if (saved && ACCENT_PALETTES[saved]) return saved
  } catch {}
  return 'indigo'
}

/** List of accent names for UI rendering (swatch buttons). */
export function getAccentNames(): string[] {
  return ACCENT_NAMES
}
"#;

    /// Convert an initial-value AuraExpr to a JS literal (v1: simple cases).
    fn store_init_to_js(expr: &crate::ast::Expr) -> String {
        use crate::ast::Expr;
        match expr {
            Expr::Int(n) => n.to_string(),
            Expr::Str(s) | Expr::CStr(s) => format!("'{}'", s.as_str().replace('\'', "\\'")),
            Expr::Bool(b) => b.to_string(),
            Expr::Array(_) => "[]".to_string(),
            _ => "null".to_string(),
        }
    }

    /// Generate Vue Router configuration file (Plan 105)
    ///
    /// Creates a `router/index.ts` file with route definitions.
    pub fn generate_router_file(routes: &[crate::aura::AuraRoute]) -> String {
        let mut route_defs = Vec::new();

        for route in routes {
            // Generate route definition with lazy loading (Plan 106)
            let module = &route.module;
            let path = &route.path;

            // Create route object with lazy loading
            if route.params.is_empty() {
                route_defs.push(format!(
                    "  {{ path: '{}', name: '{}', component: () => import('@/pages/{}.vue') }}",
                    path,
                    module,
                    module
                ));
            } else {
                // Route with params - add props: true for dynamic segments
                route_defs.push(format!(
                    "  {{ path: '{}', name: '{}', component: () => import('@/pages/{}.vue'), props: true }}",
                    path,
                    module,
                    module
                ));
            }
        }

        // No static imports needed - using lazy loading

        format!(
            r#"import {{ createRouter, createWebHashHistory }} from 'vue-router'
import type {{ RouteRecordRaw }} from 'vue-router'

const routes: RouteRecordRaw[] = [
{}
]

const router = createRouter({{
  history: createWebHashHistory(),
  routes,
}})

export default router
"#,
            route_defs.join(",\n")
        )
    }
}

impl BackendGenerator for VueGenerator {
    fn generate(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.generate_sfc(widget)
    }

    fn extension(&self) -> &'static str {
        "vue"
    }
}

impl Default for VueGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Plan 331: Library templates — self-contained per-widget SFC definitions.
// ============================================================================

/// The rendered pieces of a single primitive widget for `VueMode::Library`.
struct WidgetTemplate {
    /// Body inside `<script setup lang="ts">`.
    script: &'static str,
    /// Body inside `<template>`.
    template: &'static str,
    /// Support files beyond `index.ts` (e.g. `variants.ts`), as (name, body).
    extra_support_files: Vec<(&'static str, &'static str)>,
}

/// Convert a kebab/lower widget key (`button`) to PascalCase (`Button`).
fn pascal_case(name: &str) -> String {
    name.split('_')
        .flat_map(|part| part.split('-'))
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect()
}

/// All widget names with a library template (Plan 331). Kept in sync with
/// [`library_template`]; the CLI `auto ui list` reads this.
pub const LIBRARY_WIDGETS: &[&str] = &[
    "avatar",
    "badge",
    "button",
    "card",
    "checkbox",
    "dialog",
    "input",
    "label",
    "separator",
    "switch",
    "tabs",
    "textarea",
];

impl VueGenerator {
    /// Names of all widgets with a self-contained library template (Plan 331).
    pub const LIBRARY_WIDGETS: &'static [&'static str] = LIBRARY_WIDGETS;

    /// Plan 337: Is an AURA registry tag "covered" by the library?
    ///
    /// A tag `t` is covered ⟺ some library widget `w` satisfies
    /// `t == w` or `t.starts_with("{w}-")` (composite coverage — e.g. library
    /// `card` covers AURA `card-content`, `card-header`, …).
    pub fn covers_aura_tag(tag: &str) -> bool {
        Self::LIBRARY_WIDGETS.iter().any(|w| tag == *w || tag.starts_with(&format!("{w}-")))
    }
}

/// The `cn` class-merge helper emitted at the registry root (`registry/utils.ts`)
/// and imported by every library widget as `../utils`. Plan 331.
const LIBRARY_UTILS_TS: &str = r#"import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
"#;

/// The attribution comment prepended to every generated library SFC (Plan 331).
fn attribution_header(name: &str) -> String {
    format!(
        "<!-- Generated by AutoUI from widgets/{name}.at.\n\
         \x20    Visual layer derived from shadcn-vue (MIT). See NOTICES. -->"
    )
}

/// Look up the library template for a primitive widget name.
///
/// Phase 1.4: button / input / label. Remaining v1 widgets land in Phase 5.
fn library_template(name: &str) -> Option<WidgetTemplate> {
    match name {
        "button" => Some(WidgetTemplate {
            script: r#"import { Primitive } from 'reka-ui'
import { cn } from '../utils'
import { buttonVariants } from './variants'
import type { ButtonVariants } from './variants'

const props = withDefaults(defineProps<{
  variant?: ButtonVariants['variant']
  size?: ButtonVariants['size']
  class?: string
  as?: string
  asChild?: boolean
}>(), { variant: 'default', size: 'default', as: 'button' })"#,
            template: r#"  <Primitive :as="as" :as-child="asChild" :class="cn(buttonVariants({ variant, size }), props.class)">
    <slot />
  </Primitive>"#,
            extra_support_files: vec![(
                "variants.ts",
                r#"import { cva, type VariantProps } from 'class-variance-authority'

export const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground hover:bg-primary/90',
        destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90',
        outline: 'border border-input bg-background hover:bg-accent hover:text-accent-foreground',
        secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80',
        ghost: 'hover:bg-accent hover:text-accent-foreground',
        link: 'text-primary underline-offset-4 hover:underline',
      },
      size: {
        default: 'h-10 px-4 py-2',
        sm: 'h-9 rounded-md px-3',
        lg: 'h-11 rounded-md px-8',
        icon: 'h-10 w-10',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
)

export type ButtonVariants = VariantProps<typeof buttonVariants>
"#,
            )],
        }),
        "input" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { cn } from '../utils'

const props = defineProps<{
  defaultValue?: string | number
  modelValue?: string | number
  class?: HTMLAttributes['class']
}>()
const emits = defineEmits<{ 'update:modelValue': [value: string | number] }>()"#,
            template: r#"  <input
    :value="modelValue ?? defaultValue"
    @input="emits('update:modelValue', ($event.target as HTMLInputElement).value)"
    :class="cn('flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50', props.class)"
  />"#,
            extra_support_files: vec![],
        }),
        "label" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { Label, type LabelProps } from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<LabelProps & { class?: HTMLAttributes['class'] }>()"#,
            template: r#"  <Label
    :for="props.for"
    :class="cn('text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70', props.class)"
  >
    <slot />
  </Label>"#,
            extra_support_files: vec![],
        }),
        "textarea" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { cn } from '../utils'

const props = defineProps<{
  defaultValue?: string | number
  modelValue?: string | number
  class?: HTMLAttributes['class']
}>()
const emits = defineEmits<{ 'update:modelValue': [value: string | number] }>()"#,
            template: r#"  <textarea
    :value="modelValue ?? defaultValue"
    @input="emits('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    :class="cn('flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50', props.class)"
  />"#,
            extra_support_files: vec![],
        }),
        "checkbox" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import {
  CheckboxRoot,
  CheckboxIndicator,
  type CheckboxRootEmits,
  type CheckboxRootProps,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<CheckboxRootProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<CheckboxRootEmits>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
const forwarded = useForwardPropsEmits(delegatedProps, emits)"#,
            template: r#"  <CheckboxRoot
    v-bind="forwarded"
    :class="cn('peer h-4 w-4 shrink-0 rounded-sm border border-primary ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground', props.class)"
  >
    <CheckboxIndicator class="flex h-full w-full items-center justify-center text-current">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="4" stroke-linecap="round" stroke-linejoin="round" class="h-3.5 w-3.5"><polyline points="20 6 9 17 4 12" /></svg>
    </CheckboxIndicator>
  </CheckboxRoot>"#,
            extra_support_files: vec![],
        }),
        "switch" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import {
  SwitchRoot,
  SwitchThumb,
  type SwitchRootEmits,
  type SwitchRootProps,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<SwitchRootProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<SwitchRootEmits>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
const forwarded = useForwardPropsEmits(delegatedProps, emits)"#,
            template: r#"  <SwitchRoot
    v-bind="forwarded"
    :class="cn('peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=unchecked]:bg-input', props.class)"
  >
    <SwitchThumb class="pointer-events-none block h-5 w-5 rounded-full bg-background shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0" />
  </SwitchRoot>"#,
            extra_support_files: vec![],
        }),
        "card" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { cn } from '../utils'

const props = defineProps<{ class?: HTMLAttributes['class'] }>()"#,
            template: r#"  <div :class="cn('rounded-lg border bg-card text-card-foreground shadow-sm', props.class)">
    <slot />
  </div>"#,
            extra_support_files: vec![
                ("CardHeader.vue", "<!-- Generated by AutoUI from widgets/card.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { cn } from '../utils'\n\nconst props = defineProps<{ class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <div :class=\"cn('flex flex-col space-y-1.5 p-6', props.class)\"><slot /></div>\n</template>\n"),
                ("CardTitle.vue", "<!-- Generated by AutoUI from widgets/card.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { cn } from '../utils'\n\nconst props = defineProps<{ class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <h3 :class=\"cn('text-2xl font-semibold leading-none tracking-tight', props.class)\"><slot /></h3>\n</template>\n"),
                ("CardDescription.vue", "<!-- Generated by AutoUI from widgets/card.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { cn } from '../utils'\n\nconst props = defineProps<{ class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <p :class=\"cn('text-sm text-muted-foreground', props.class)\"><slot /></p>\n</template>\n"),
                ("CardContent.vue", "<!-- Generated by AutoUI from widgets/card.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { cn } from '../utils'\n\nconst props = defineProps<{ class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <div :class=\"cn('p-6 pt-0', props.class)\"><slot /></div>\n</template>\n"),
                ("CardFooter.vue", "<!-- Generated by AutoUI from widgets/card.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { cn } from '../utils'\n\nconst props = defineProps<{ class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <div :class=\"cn('flex items-center p-6 pt-0', props.class)\"><slot /></div>\n</template>\n"),
            ],
        }),
        "separator" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { Separator, type SeparatorProps, useForwardProps } from 'reka-ui'
import { cn } from '../utils'

const props = withDefaults(
  defineProps<SeparatorProps & { class?: HTMLAttributes['class'] }>(),
  { orientation: 'horizontal', decorative: true },
)

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
const forwarded = useForwardProps(delegatedProps)"#,
            template: r#"  <Separator
    v-bind="forwarded"
    :class="cn('shrink-0 bg-border', props.orientation === 'vertical' ? 'h-full w-[1px]' : 'h-[1px] w-full', props.class)"
  />"#,
            extra_support_files: vec![],
        }),
        "badge" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { cn } from '../utils'
import { badgeVariants, type BadgeVariants } from './variants'

const props = defineProps<{
  variant?: BadgeVariants['variant']
  class?: HTMLAttributes['class']
}>()"#,
            template: r#"  <div :class="cn(badgeVariants({ variant: props.variant }), props.class)">
    <slot />
  </div>"#,
            extra_support_files: vec![(
                "variants.ts",
                r#"import { cva, type VariantProps } from 'class-variance-authority'

export const badgeVariants = cva(
  'inline-flex items-center rounded-md border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
  {
    variants: {
      variant: {
        default: 'border-transparent bg-primary text-primary-foreground hover:bg-primary/80',
        secondary: 'border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80',
        destructive: 'border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80',
        outline: 'text-foreground',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
)

export type BadgeVariants = VariantProps<typeof badgeVariants>
"#,
            )],
        }),
        "avatar" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { AvatarRoot, type AvatarRootProps, useForwardProps } from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<AvatarRootProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
const forwarded = useForwardProps(delegatedProps)"#,
            template: r#"  <AvatarRoot v-bind="forwarded" :class="cn('relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full', props.class)">
    <slot />
  </AvatarRoot>"#,
            extra_support_files: vec![
                ("AvatarImage.vue", "<!-- Generated by AutoUI from widgets/avatar.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { AvatarImage, type AvatarImageProps } from 'reka-ui'\n\nconst props = defineProps<AvatarImageProps & { class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <AvatarImage v-bind=\"props\" class=\"aspect-square h-full w-full\" />\n</template>\n"),
                ("AvatarFallback.vue", "<!-- Generated by AutoUI from widgets/avatar.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { AvatarFallback, type AvatarFallbackProps } from 'reka-ui'\nimport { cn } from '../utils'\n\nconst props = defineProps<AvatarFallbackProps & { class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <AvatarFallback v-bind=\"props\" :class=\"cn('flex h-full w-full items-center justify-center rounded-full bg-muted', props.class)\"><slot /></AvatarFallback>\n</template>\n"),
            ],
        }),
        "dialog" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import {
  DialogRoot,
  DialogTrigger,
  type DialogRootEmits,
  type DialogRootProps,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<DialogRootProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<DialogRootEmits>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
const forwarded = useForwardPropsEmits(delegatedProps, emits)"#,
            template: r#"  <DialogRoot v-bind="forwarded">
    <DialogTrigger v-if="$slots.trigger" as-child><slot name="trigger" /></DialogTrigger>
    <slot />
  </DialogRoot>"#,
            extra_support_files: vec![
                ("DialogContent.vue", r#"<!-- Generated by AutoUI from widgets/dialog.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import {
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogTitle,
  type DialogContentEmits,
  type DialogContentProps,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<DialogContentProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<DialogContentEmits>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <DialogPortal>
    <DialogOverlay class="fixed inset-0 z-50 bg-black/80 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0" />
    <DialogContent
      v-bind="forwarded"
      :class="cn('fixed left-1/2 top-1/2 z-50 grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 border bg-background p-6 shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 sm:rounded-lg', props.class)"
    >
      <slot />
      <DialogTitle v-if="$slots.title" as-child><slot name="title" /></DialogTitle>
      <DialogDescription v-if="$slots.description" as-child><slot name="description" /></DialogDescription>
      <DialogClose class="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="h-4 w-4"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
        <span class="sr-only">Close</span>
      </DialogClose>
    </DialogContent>
  </DialogPortal>
</template>
"#),
                ("DialogHeader.vue", "<!-- Generated by AutoUI from widgets/dialog.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { cn } from '../utils'\n\nconst props = defineProps<{ class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <div :class=\"cn('flex flex-col space-y-1.5 text-center sm:text-left', props.class)\"><slot /></div>\n</template>\n"),
                ("DialogFooter.vue", "<!-- Generated by AutoUI from widgets/dialog.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { cn } from '../utils'\n\nconst props = defineProps<{ class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <div :class=\"cn('flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2', props.class)\"><slot /></div>\n</template>\n"),
            ],
        }),
        "tabs" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import {
  TabsRoot,
  type TabsRootEmits,
  type TabsRootProps,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<TabsRootProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<TabsRootEmits>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
const forwarded = useForwardPropsEmits(delegatedProps, emits)"#,
            template: r#"  <TabsRoot v-bind="forwarded" :class="cn('relative', props.class)">
    <slot />
  </TabsRoot>"#,
            extra_support_files: vec![
                ("TabsList.vue", "<!-- Generated by AutoUI from widgets/tabs.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { TabsList, type TabsListProps } from 'reka-ui'\nimport { cn } from '../utils'\n\nconst props = defineProps<TabsListProps & { class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <TabsList v-bind=\"props\" :class=\"cn('inline-flex h-10 items-center justify-center rounded-md bg-muted p-1 text-muted-foreground', props.class)\" />\n</template>\n"),
                ("TabsTrigger.vue", "<!-- Generated by AutoUI from widgets/tabs.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { computed } from 'vue'\nimport { TabsTrigger, type TabsTriggerProps, useForwardProps } from 'reka-ui'\nimport { cn } from '../utils'\n\nconst props = defineProps<TabsTriggerProps & { class?: HTMLAttributes['class'] }>()\nconst delegatedProps = computed(() => { const { class: _, ...d } = props; return d })\nconst forwarded = useForwardProps(delegatedProps)\n</script>\n\n<template>\n  <TabsTrigger v-bind=\"forwarded\" :class=\"cn('inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm', props.class)\" />\n</template>\n"),
                ("TabsContent.vue", "<!-- Generated by AutoUI from widgets/tabs.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { computed } from 'vue'\nimport { TabsContent, type TabsContentProps, useForwardProps } from 'reka-ui'\nimport { cn } from '../utils'\n\nconst props = defineProps<TabsContentProps & { class?: HTMLAttributes['class'] }>()\nconst delegatedProps = computed(() => { const { class: _, ...d } = props; return d })\nconst forwarded = useForwardProps(delegatedProps)\n</script>\n\n<template>\n  <TabsContent v-bind=\"forwarded\" :class=\"cn('mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2', props.class)\" />\n</template>\n"),
            ],
        }),
        _ => None,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aura::{AuraMessage, AuraMsgVariant, AuraStateDef};
    use std::collections::HashMap;

    #[test]
    fn test_vue_generator_creation() {
        let gen = VueGenerator::new();
        assert!(gen.current_widget.is_none());
    }

    #[test]
    fn test_simple_counter() {
        let widget = AuraWidget {
            name: "Counter".to_string(),
            state_vars: vec![AuraStateDef {
                name: "count".to_string(),
                type_info: crate::ast::Type::Int,
                initial: crate::ast::Expr::Int(0),
                decorators: vec![],
            }],
            messages: vec![AuraMessage {
                name: "Msg".to_string(),
                variants: vec![
                    AuraMsgVariant { name: "Inc".to_string(), payload: vec![] },
                    AuraMsgVariant { name: "Dec".to_string(), payload: vec![] },
                ],
            }],
            view_tree: AuraNode::element("col")
                .with_child(AuraNode::text("Count: 0")),
            handlers: HashMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
        }
;

        let mut gen = VueGenerator::new();
        let sfc = gen.generate(&widget).unwrap();

        // Plan 100: Default is now TypeScript, so check for lang="ts"
        assert!(sfc.contains(r#"<script setup lang="ts">"#));
        assert!(sfc.contains("import { ref } from 'vue'"));
        assert!(sfc.contains("const count = ref<number>(0)"));
        assert!(sfc.contains("<template>"));
        assert!(sfc.contains("<style>"));
    }

    /// Widget-level native CSS (`style { ... }` → `AuraWidget.style_css`) is
    /// emitted verbatim into a dedicated `<style scoped>` block.
    #[test]
    fn test_widget_style_css_scoped_passthrough() {
        let css = "\n.autodown-editor {\n  --ad-border: #333;\n}\n.autodown-editor:hover {\n  border-color: var(--ad-border);\n}\n@media (max-width: 768px) {\n  .autodown-editor {\n    font-size: 12px;\n  }\n}\n";
        let widget = AuraWidget {
            name: "Styled".to_string(),
            state_vars: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col")
                .with_child(AuraNode::text("hi")),
            handlers: HashMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: Some(css.to_string()),
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
        };

        let mut gen = VueGenerator::new();
        let sfc = gen.generate(&widget).unwrap();

        assert!(sfc.contains("<style scoped>"), "sfc:\n{}", sfc);
        // The CSS body is byte-for-byte identical inside the scoped block.
        assert!(sfc.contains(css), "scoped css not verbatim in sfc:\n{}", sfc);
        assert!(sfc.contains(".autodown-editor:hover"));
        assert!(sfc.contains("@media (max-width: 768px) {"));
        // The plain generated <style> block is still present.
        assert!(sfc.contains("<style>"));
    }

    /// Template ref escape hatch: a `ref` prop on a view element emits a
    /// static `ref="menuEl"` template attribute plus a
    /// `const menuEl = ref<HTMLElement | null>(null)` script declaration,
    /// and pulls in the `ref` import even without state vars.
    #[test]
    fn test_template_ref_declaration() {
        let widget = AuraWidget {
            name: "Menu".to_string(),
            state_vars: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col")
                .with_prop("ref", crate::ast::Expr::Str("menuEl".into()))
                .with_child(AuraNode::text("hi")),
            handlers: HashMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
        };

        let mut gen = VueGenerator::new();
        let sfc = gen.generate(&widget).unwrap();

        assert!(sfc.contains("import { ref } from 'vue'"), "sfc:\n{}", sfc);
        assert!(
            sfc.contains("const menuEl = ref<HTMLElement | null>(null)"),
            "sfc:\n{}",
            sfc
        );
        assert!(sfc.contains("ref=\"menuEl\""), "sfc:\n{}", sfc);
    }

    /// Without a style block, no `<style scoped>` is emitted.
    #[test]
    fn test_no_widget_style_css_no_scoped_block() {
        let widget = AuraWidget {
            name: "Plain".to_string(),
            state_vars: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col")
                .with_child(AuraNode::text("hi")),
            handlers: HashMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
        };

        let mut gen = VueGenerator::new();
        let sfc = gen.generate(&widget).unwrap();
        assert!(!sfc.contains("<style scoped>"), "sfc:\n{}", sfc);
    }

    /// Plan 354 Phase C: AutoDownEditor renders as a Vue component with
    /// camelCase prop bindings, @autodown/editor named import, and events
    /// mapped to @update / @save / @cancel.
    #[test]
    fn test_autodown_editor_rendering() {
        use crate::ast::Expr;
        let widget = AuraWidget {
            name: "NoteEditor".to_string(),
            state_vars: vec![AuraStateDef {
                name: "body".to_string(),
                type_info: crate::ast::Type::StrSlice,
                initial: Expr::Str("# Welcome".into()),
                decorators: vec![],
            }],
            messages: vec![AuraMessage {
                name: "Msg".to_string(),
                variants: vec![AuraMsgVariant {
                    name: "BodyChanged".to_string(),
                    payload: vec![],
                }],
            }],
            // autodown_editor { content: .body; onupdate: .BodyChanged; style: "..." }
            view_tree: AuraNode::element("col").with_child(
                AuraNode::element("autodown_editor")
                    .with_prop("content", Expr::Ident(".body".into()))
                    .with_event("onupdate", ".BodyChanged"),
            ),
            handlers: HashMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
        };

        let mut gen = VueGenerator::new().with_mode(VueMode::Shadcn);
        let sfc = gen.generate(&widget).unwrap();

        // Renders as PascalCase component, not <div> or <autodown_editor>.
        assert!(sfc.contains("<AutoDownEditor"), "tag is AutoDownEditor:\n{}", sfc);
        // content bound to the body state ref.
        assert!(
            sfc.contains(":content=\"body\""),
            "content bound to body:\n{}",
            sfc
        );
        // Defaults: can-edit and show-actions default to true.
        assert!(sfc.contains(":can-edit=\"true\""), "can_edit defaults true:\n{}", sfc);
        assert!(sfc.contains(":show-actions=\"true\""), "show_actions defaults true:\n{}", sfc);
        // Event: onupdate → @update with the dot-handler name.
        assert!(
            sfc.contains("@update=\"BodyChanged\""),
            "onupdate → @update:\n{}",
            sfc
        );
        // Named import from @autodown/editor.
        assert!(
            sfc.contains("import { AutoDownEditor } from '@autodown/editor'"),
            "named import from @autodown/editor:\n{}",
            sfc
        );
    }

    #[test]
    fn test_autodown_editor_props_camelcase_and_events() {
        use crate::ast::Expr;
        // Verify bool literals map through and onsave/oncancel events resolve.
        let widget = AuraWidget {
            name: "NoteEditor".to_string(),
            state_vars: vec![AuraStateDef {
                name: "note_body".to_string(),
                type_info: crate::ast::Type::StrSlice,
                initial: Expr::Str("".into()),
                decorators: vec![],
            }],
            messages: vec![AuraMessage {
                name: "Msg".to_string(),
                variants: vec![
                    AuraMsgVariant { name: "Save".to_string(), payload: vec![] },
                    AuraMsgVariant { name: "Cancel".to_string(), payload: vec![] },
                ],
            }],
            view_tree: AuraNode::element("col").with_child(
                AuraNode::element("autodown_editor")
                    .with_prop("content", Expr::Ident(".note_body".into()))
                    .with_prop("can_edit", Expr::Bool(false))
                    .with_prop("show_actions", Expr::Bool(true))
                    .with_event("onsave", ".Save")
                    .with_event("oncancel", ".Cancel"),
            ),
            handlers: HashMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
        };

        let mut gen = VueGenerator::new().with_mode(VueMode::Shadcn);
        let sfc = gen.generate(&widget).unwrap();

        // Field-access content binding: .note_body → note_body.
        assert!(sfc.contains(":content=\"note_body\""), "field access content:\n{}", sfc);
        // Bool literals propagate.
        assert!(sfc.contains(":can-edit=\"false\""), "can_edit false:\n{}", sfc);
        assert!(sfc.contains(":show-actions=\"true\""), "show_actions true:\n{}", sfc);
        // onsave/oncancel → @save/@cancel.
        assert!(sfc.contains("@save=\"Save\""), "onsave → @save:\n{}", sfc);
        assert!(sfc.contains("@cancel=\"Cancel\""), "oncancel → @cancel:\n{}", sfc);
    }

    #[test]
    fn test_expr_to_js() {
        let gen = VueGenerator::new();

        assert_eq!(gen.expr_to_js(&crate::ast::Expr::Int(42)).unwrap(), "42");
        assert_eq!(gen.expr_to_js(&crate::ast::Expr::Bool(true)).unwrap(), "true");
        assert_eq!(gen.expr_to_js(&crate::ast::Expr::Str("hello".into())).unwrap(), "'hello'");
    }

    #[test]
    fn test_map_tag() {
        let mut gen = VueGenerator::new();

        assert_eq!(gen.map_tag("col", true), "div");
        assert_eq!(gen.map_tag("button", false), "button");
        assert_eq!(gen.map_tag("h2", false), "h2");
    }

    #[test]
    fn test_map_tag_hyphenated() {
        let mut gen = VueGenerator::new();

        // Hyphenated tags should pass through correctly
        assert_eq!(gen.map_tag("preview-card", false), "div");
        assert_eq!(gen.map_tag("preview-card", true), "div");

        // Both previewcard and preview-card should map to the same thing
        assert_eq!(gen.map_tag("previewcard", false), gen.map_tag("preview-card", false));

        // Other hyphenated tags (fallback to div for unknown)
        assert_eq!(gen.map_tag("my-custom-tag", false), "div");

        // Known tags with hyphens in HTML5
        // (these would pass through if added to the match)
    }

    #[test]
    fn test_pattern_to_handler_name() {
        let gen = VueGenerator::new();

        assert_eq!(gen.pattern_to_handler_name("Msg::Inc"), "onInc");
        assert_eq!(gen.pattern_to_handler_name(".Inc"), "Inc");
        assert_eq!(gen.pattern_to_handler_name(".openSidebar"), "openSidebar");
        assert_eq!(gen.pattern_to_handler_name("Dec"), "onDec");
    }

    #[test]
    fn test_shadcn_mode() {
        let gen = VueGenerator::new_shadcn();
        assert!(gen.is_shadcn());

        let gen = VueGenerator::new().with_mode(VueMode::Shadcn);
        assert!(gen.is_shadcn());

        let gen = VueGenerator::new();
        assert!(!gen.is_shadcn());
    }

    #[test]
    fn test_library_mode_constructor() {
        let gen = VueGenerator::new_library();
        assert!(gen.is_library());
        assert!(!gen.is_shadcn());

        let gen = VueGenerator::new().with_mode(VueMode::Library);
        assert!(gen.is_library());
    }

    #[test]
    fn test_library_button_sfc_is_self_contained() {
        let mut gen = VueGenerator::new_library();
        let sfc = gen.generate_widget_sfc("button").unwrap();
        assert!(sfc.contains("<template>"), "has template");
        assert!(sfc.contains("<script setup"), "has script setup");
        assert!(!sfc.contains("@/components/ui/"), "must NOT import shadcn-vue");
        assert!(sfc.contains("reka-ui"), "uses reka-ui as backend");
    }

    #[test]
    fn test_library_button_support_files() {
        let gen = VueGenerator::new_library();
        let files = gen.generate_widget_support_files("button");
        let names: Vec<&str> = files.iter().map(|(p, _)| p.as_str()).collect();
        assert!(names.contains(&"variants.ts"), "variants.ts present: {:?}", names);
        assert!(names.contains(&"index.ts"), "index.ts present: {:?}", names);
        let index = files.iter().find(|(p, _)| p == "index.ts").unwrap();
        assert!(index.1.contains("Button"), "index re-exports Button");
    }

    #[test]
    fn test_library_input_sfc_is_self_contained() {
        let mut gen = VueGenerator::new_library();
        let sfc = gen.generate_widget_sfc("input").unwrap();
        assert!(sfc.contains("<template>"), "has template");
        assert!(sfc.contains("<script setup"), "has script setup");
        assert!(!sfc.contains("@/components/ui/"), "must NOT import shadcn-vue");
    }

    #[test]
    fn test_library_label_sfc_uses_reka_ui() {
        let mut gen = VueGenerator::new_library();
        let sfc = gen.generate_widget_sfc("label").unwrap();
        assert!(sfc.contains("<template>"), "has template");
        assert!(sfc.contains("<script setup"), "has script setup");
        assert!(sfc.contains("reka-ui"), "label uses reka-ui Label");
        assert!(!sfc.contains("@/components/ui/"), "must NOT import shadcn-vue");
    }

    #[test]
    fn test_library_unknown_widget_errors() {
        let mut gen = VueGenerator::new_library();
        let err = gen.generate_widget_sfc("does-not-exist").unwrap_err();
        assert!(format!("{err}").contains("Unknown widget"), "got: {err}");
    }

    #[test]
    fn test_library_sfc_has_attribution_header() {
        let mut gen = VueGenerator::new_library();
        let sfc = gen.generate_widget_sfc("button").unwrap();
        assert!(
            sfc.starts_with("<!-- Generated by AutoUI"),
            "must start with attribution header: {}",
            sfc.lines().next().unwrap_or("")
        );
        assert!(sfc.contains("shadcn-vue (MIT)"), "must cite shadcn-vue (MIT)");
        assert!(sfc.contains("NOTICES"), "must point to NOTICES");
    }

    #[test]
    fn test_library_all_widgets_self_contained() {
        let mut gen = VueGenerator::new_library();
        for name in VueGenerator::LIBRARY_WIDGETS {
            let sfc = gen.generate_widget_sfc(name).unwrap_or_else(|e| panic!("generate {name}: {e}"));
            assert!(sfc.contains("<template>"), "{name}: has template");
            assert!(sfc.contains("<script setup"), "{name}: has script setup");
            assert!(!sfc.contains("@/components/ui/"), "{name}: self-contained");
            assert!(
                sfc.starts_with("<!-- Generated by AutoUI"),
                "{name}: attribution header"
            );
        }
    }

    #[test]
    fn test_library_reka_ui_backed_widgets() {
        let mut gen = VueGenerator::new_library();
        // widget -> a marker that proves it binds the right reka-ui primitive.
        let markers: &[(&str, &str)] = &[
            ("checkbox", "CheckboxRoot"),
            ("switch", "SwitchRoot"),
            ("separator", "<Separator"),
            ("avatar", "AvatarRoot"),
            ("dialog", "DialogRoot"),
            ("tabs", "TabsRoot"),
            ("label", "Label"),
        ];
        for (name, marker) in markers {
            let sfc = gen.generate_widget_sfc(name).unwrap();
            assert!(sfc.contains(marker), "{name}: should use {marker}");
        }
    }

    #[test]
    fn test_library_composite_widget_support_files() {
        let gen = VueGenerator::new_library();
        // card ships 5 companion SFCs.
        let card_files: Vec<String> =
            gen.generate_widget_support_files("card").into_iter().map(|(n, _)| n).collect();
        for companion in [
            "index.ts",
            "CardHeader.vue",
            "CardTitle.vue",
            "CardDescription.vue",
            "CardContent.vue",
            "CardFooter.vue",
        ] {
            assert!(card_files.contains(&companion.to_string()), "card missing {companion}");
        }
        // tabs ships 3 companion SFCs.
        let tabs_files: Vec<String> =
            gen.generate_widget_support_files("tabs").into_iter().map(|(n, _)| n).collect();
        for companion in ["index.ts", "TabsList.vue", "TabsTrigger.vue", "TabsContent.vue"] {
            assert!(tabs_files.contains(&companion.to_string()), "tabs missing {companion}");
        }
    }

    #[test]
    fn test_library_index_reexports_all_vue_files() {
        let gen = VueGenerator::new_library();
        let files = gen.generate_widget_support_files("card");
        let index = files.iter().find(|(n, _)| n == "index.ts").unwrap();
        // primary + 5 companions = 6 re-exports
        assert_eq!(index.1.matches("export").count(), 6, "index: {}", index.1);
        assert!(index.1.contains("Card"), "re-exports Card");
        assert!(index.1.contains("CardHeader"), "re-exports CardHeader");
    }

    /// Plan 337: pin LIBRARY_WIDGETS self-consistency — every advertised
    /// widget must have a renderable template, and the list must be sorted.
    #[test]
    fn test_library_widgets_list_is_self_consistent() {
        for name in VueGenerator::LIBRARY_WIDGETS {
            let mut gen = VueGenerator::new_library();
            gen.generate_widget_sfc(name)
                .unwrap_or_else(|e| panic!("LIBRARY_WIDGETS lists '{name}' but template is missing: {e}"));
        }
        let mut sorted = VueGenerator::LIBRARY_WIDGETS.to_vec();
        sorted.sort();
        assert_eq!(
            VueGenerator::LIBRARY_WIDGETS.to_vec(), sorted,
            "LIBRARY_WIDGETS must be sorted"
        );
    }

    /// Plan 337 Task 2.1: drift guard — every library widget must exist in
    /// the AURA registry (exact tag or prefix-grouped composite).
    #[test]
    fn test_library_widgets_exist_in_aura_registry() {
        let reg = WidgetRegistry::with_defaults();
        let aura_tags: std::collections::HashSet<&str> =
            reg.all_widgets().keys().map(|s| s.as_str()).collect();
        for w in VueGenerator::LIBRARY_WIDGETS {
            let known = aura_tags.contains(*w)
                || aura_tags.iter().any(|t| t.starts_with(&format!("{w}-")));
            assert!(
                known,
                "LIBRARY_WIDGETS has '{w}' but AURA registry has no such widget"
            );
        }
    }

    /// Plan 337 Task 2.1: covers_aura_tag composite coverage logic.
    #[test]
    fn test_covers_aura_tag_composite() {
        // Exact match
        assert!(VueGenerator::covers_aura_tag("button"));
        assert!(VueGenerator::covers_aura_tag("card"));
        // Composite (prefix-dash)
        assert!(VueGenerator::covers_aura_tag("card-content"));
        assert!(VueGenerator::covers_aura_tag("card-header"));
        // Non-covered
        assert!(!VueGenerator::covers_aura_tag("accordion"));
        assert!(!VueGenerator::covers_aura_tag("data-table"));
    }

    #[test]
    fn test_shadcn_map_tag() {
        let mut gen = VueGenerator::new_shadcn();

        // Should return shadcn component names
        assert_eq!(gen.map_tag("button", false), "Button");
        assert_eq!(gen.map_tag("input", true), "Input");
        assert_eq!(gen.map_tag("textarea", true), "Textarea");
        assert_eq!(gen.map_tag("checkbox", true), "Checkbox");
        assert_eq!(gen.map_tag("toggle", true), "Switch");
        assert_eq!(gen.map_tag("select", false), "Select");
        assert_eq!(gen.map_tag("progress", true), "Progress");
        assert_eq!(gen.map_tag("badge", true), "Badge");
        assert_eq!(gen.map_tag("card", false), "Card");
        assert_eq!(gen.map_tag("avatar", true), "Avatar");
        assert_eq!(gen.map_tag("slider", true), "Slider");

        // Layout elements should still return div
        assert_eq!(gen.map_tag("col", false), "div");
        assert_eq!(gen.map_tag("row", false), "div");
    }

    #[test]
    fn test_shadcn_registry() {
        let registry = ShadcnRegistry::new();

        // Check component mappings exist
        assert!(registry.has_component("button"));
        assert!(registry.has_component("input"));
        assert!(registry.has_component("checkbox"));
        assert!(registry.has_component("modal"));
        assert!(registry.has_component("tabs"));
        assert!(registry.has_component("table"));

        // Check primary component names
        assert_eq!(registry.primary_component("button"), Some("Button"));
        assert_eq!(registry.primary_component("input"), Some("Input"));
        assert_eq!(registry.primary_component("toggle"), Some("Switch"));
    }

    #[test]
    fn test_generate_shadcn_attrs_area_chart() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        props.insert("data".to_string(), AuraPropValue::Expr(crate::ast::Expr::Ident("monthlyRevenue".into())));
        props.insert("categories".to_string(), AuraPropValue::Expr(crate::ast::Expr::Array(vec![
            crate::ast::Expr::Str("desktop".into()),
            crate::ast::Expr::Str("mobile".into()),
        ])));
        props.insert("index".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("month".into())));
        props.insert("show-x-axis".to_string(), AuraPropValue::Expr(crate::ast::Expr::Bool(false)));
        let (attrs, _, _) = gen.generate_shadcn_attrs("area-chart", &props, &events);

        assert!(attrs.iter().any(|a| a.contains(":data=\"monthlyRevenue\"")));
        assert!(attrs.iter().any(|a| a.contains(":categories=")));
        assert!(attrs.iter().any(|a| a.contains("index=\"month\"")));
        assert!(attrs.iter().any(|a| a.contains(":show-x-axis=\"false\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_bar_chart() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        props.insert("data".to_string(), AuraPropValue::Expr(crate::ast::Expr::Ident("quarterlySales".into())));
        props.insert("type".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("stacked".into())));
        props.insert("rounded-corners".to_string(), AuraPropValue::Expr(crate::ast::Expr::Bool(true)));
        let (attrs, _, _) = gen.generate_shadcn_attrs("bar-chart", &props, &events);

        assert!(attrs.iter().any(|a| a.contains(":data=\"quarterlySales\"")));
        assert!(attrs.iter().any(|a| a.contains("type=\"stacked\"")));
        assert!(attrs.iter().any(|a| a.contains(":rounded-corners=\"true\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_line_chart_with_curve() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        props.insert("curve-type".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("monotone".into())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("line-chart", &props, &events);

        assert!(attrs.iter().any(|a| a.contains(":curve-type=\"CurveType.MonotoneX\"")));
        assert!(gen.use_curve_type);
    }

    #[test]
    fn test_generate_shadcn_attrs_donut_chart() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        props.insert("category".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("source".into())));
        props.insert("value-formatter".to_string(), AuraPropValue::Expr(crate::ast::Expr::Ident("formatValue".into())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("donut-chart", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("category=\"source\"")));
        assert!(attrs.iter().any(|a| a.contains(":value-formatter=\"formatValue\"")));
    }

    #[test]
    fn test_dashboard_01_compiles() {
        use crate::ui_build_shadcn;
        let result = ui_build_shadcn("../../examples/gallery/source/front/pages/blocks/dashboard_01.at", None);
        assert!(result.is_ok(), "dashboard_01 should compile: {:?}", result.err());
        let code = result.unwrap();
        assert!(code.contains("<AreaChart"), "AreaChart tag missing in dashboard");
        assert!(code.contains(":data=\"revenueData\""), "revenueData binding missing");
        assert!(code.contains("index=\"month\""), "month index missing");
    }

    #[test]
    fn test_charts_gallery_compiles() {
        // Integration test: compile the charts gallery app.at and verify output
        use crate::ui_build_shadcn;
        let result = ui_build_shadcn("../../examples/charts-gallery/src/front/app.at", None);
        assert!(result.is_ok(), "charts gallery should compile: {:?}", result.err());
        let code = result.unwrap();

        // Verify chart component tags are present
        assert!(code.contains("<AreaChart"), "AreaChart tag missing");
        assert!(code.contains("<BarChart"), "BarChart tag missing");
        assert!(code.contains("<LineChart"), "LineChart tag missing");
        assert!(code.contains("<DonutChart"), "DonutChart tag missing");

        // Verify chart imports are present
        assert!(code.contains("@/components/ui/chart-area"), "chart-area import missing");
        assert!(code.contains("@/components/ui/chart-bar"), "chart-bar import missing");
        assert!(code.contains("@/components/ui/chart-line"), "chart-line import missing");
        assert!(code.contains("@/components/ui/chart-donut"), "chart-donut import missing");

        // Verify key props are emitted
        assert!(code.contains(":data=\"monthlyRevenue\""), "monthlyRevenue data binding missing");
        assert!(code.contains("index=\"month\""), "month index missing");
        assert!(code.contains("type=\"stacked\""), "stacked type missing");
        assert!(code.contains(":curve-type=\"CurveType.MonotoneX\""), "curve type missing");
        assert!(code.contains("category=\"source\""), "donut category missing");
        assert!(code.contains(":colors="), "colors binding missing");

        // Verify CurveType import
        assert!(code.contains("import { CurveType } from '@unovis/ts'"), "CurveType import missing");
    }

    #[test]
    fn test_generate_shadcn_attrs_button() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test button with text
        props.insert("text".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("Click me".into())));
        let (attrs, _slot_content, slot_children) = gen.generate_shadcn_attrs("button", &props, &events);

        assert!(slot_children.is_some());
        assert!(slot_children.unwrap().contains("Click me"));
    }

    #[test]
    fn test_generate_shadcn_attrs_input() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test input with v-model
        props.insert("value".to_string(), AuraPropValue::Expr(crate::ast::Expr::Ident("name".into())));
        props.insert("placeholder".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("Enter name".into())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("input", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("v-model=\"name\"")));
        assert!(attrs.iter().any(|a| a.contains("placeholder=\"Enter name\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_checkbox() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test checkbox with v-model (reka-ui uses modelValue, not checked)
        props.insert("checked".to_string(), AuraPropValue::Expr(crate::ast::Expr::Ident("done".into())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("checkbox", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("v-model=\"done\"")));
    }

    #[test]
    fn test_generate_project_files() {
        // Test scaffold file generation
        let components_json = VueGenerator::generate_components_json();
        assert!(components_json.contains("shadcn-vue"));
        assert!(components_json.contains("tailwind"));

        let package_json = VueGenerator::generate_package_json("test-project");
        assert!(package_json.contains("test-project"));
        assert!(package_json.contains("radix-vue"));
        assert!(package_json.contains("tailwind-merge"));

        let vite_config = VueGenerator::generate_vite_config();
        assert!(vite_config.contains("@vitejs/plugin-vue"));
        assert!(vite_config.contains("alias"));

        let utils_ts = VueGenerator::generate_utils_ts();
        assert!(utils_ts.contains("cn"));
        assert!(utils_ts.contains("clsx"));
        assert!(utils_ts.contains("tailwind-merge"));

        let base_css = VueGenerator::generate_base_css();
        assert!(base_css.contains("--background"));
        assert!(base_css.contains("--primary"));
    }

    // ========================================
    // Phase 3: Layout & Navigation Tests
    // ========================================

    #[test]
    fn test_generate_shadcn_attrs_scroll() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test scroll area with orientation
        props.insert("orientation".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("vertical".into())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("scroll", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("orientation=\"vertical\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_tabs() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tabs with default value
        props.insert("default".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("tab1".into())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("tabs", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("default-value=\"tab1\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_tabs_with_model() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tabs with v-model
        props.insert("value".to_string(), AuraPropValue::Expr(crate::ast::Expr::Ident("activeTab".into())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("tabs", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("v-model=\"activeTab\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_tab() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tab trigger with value and text
        props.insert("value".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("tab1".into())));
        props.insert("text".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("First Tab".into())));
        let (attrs, slot_content, _) = gen.generate_shadcn_attrs("tab", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("value=\"tab1\"")));
        assert!(slot_content.is_some());
        assert_eq!(slot_content.unwrap(), "First Tab");
    }

    #[test]
    fn test_generate_shadcn_attrs_card() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test card with variant and title
        props.insert("variant".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("outline".into())));
        props.insert("title".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("Card Title".into())));
        let (attrs, slot_content, _) = gen.generate_shadcn_attrs("card", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("variant=\"outline\"")));
        assert!(slot_content.is_some());
        assert_eq!(slot_content.unwrap(), "Card Title");
    }

    #[test]
    fn test_generate_shadcn_attrs_divider() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test separator with orientation
        props.insert("orientation".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("vertical".into())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("divider", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("orientation=\"vertical\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_divider_decorative() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test decorative separator
        props.insert("decorative".to_string(), AuraPropValue::Expr(crate::ast::Expr::Bool(true)));
        let (attrs, _, _) = gen.generate_shadcn_attrs("divider", &props, &events);

        assert!(attrs.iter().any(|a| a == "decorative"));
    }

    #[test]
    fn test_shadcn_registry_phase3_components() {
        let registry = ShadcnRegistry::new();

        // Check Phase 3 component mappings
        assert!(registry.has_component("scroll"));
        assert!(registry.has_component("tabs"));
        assert!(registry.has_component("tab"));
        assert!(registry.has_component("card"));
        assert!(registry.has_component("divider"));

        // Check primary component names
        assert_eq!(registry.primary_component("scroll"), Some("ScrollArea"));
        assert_eq!(registry.primary_component("tabs"), Some("Tabs"));
        assert_eq!(registry.primary_component("tab"), Some("TabsTrigger"));
        assert_eq!(registry.primary_component("card"), Some("Card"));
        assert_eq!(registry.primary_component("divider"), Some("Separator"));

        // Check imports are returned correctly
        let (module, components) = registry.get("scroll").unwrap();
        assert!(module.contains("scroll-area"));
        assert!(components.contains(&"ScrollArea"));

        let (module, components) = registry.get("tabs").unwrap();
        assert!(module.contains("tabs"));
        assert!(components.contains(&"Tabs"));
        assert!(components.contains(&"TabsList"));
        assert!(components.contains(&"TabsTrigger"));
        assert!(components.contains(&"TabsContent"));
    }

    // ========================================
    // Phase 4: Overlay & Feedback Tests
    // ========================================

    #[test]
    fn test_generate_shadcn_attrs_modal() {
        // Build the props through the REAL parse path (widget source → parser →
        // aura extract → shadcn attrs). The previous version hand-built
        // `Expr::Ident("showDialog")`, which no real `.state` ref ever produces
        // (dot_item yields Dot(Ident("self"), name)) — so the test stayed green
        // while v-model:open was silently dropped for real widgets.
        let sfc = gen_sfc_from_widget_src_shadcn(r#"
widget ConfirmDialog {
    model { var showDialog bool = false }
    view {
        modal(open: .showDialog, title: "Confirm Delete") {
            text "Are you sure?"
        }
    }
}
"#);
        assert!(
            sfc.contains("<Dialog v-model:open=\"showDialog\""),
            "modal routed to <Dialog> with v-model:open:\n{}",
            sfc
        );
        assert!(
            sfc.contains("data-title=\"Confirm Delete\""),
            "title preserved:\n{}",
            sfc
        );
    }

    /// Item 1 regression: builtin `dialog` tag routed through the shadcn
    /// Dialog component keeps its v-model:open binding (real parse path).
    #[test]
    fn test_dialog_vmodel_open_real_dsl() {
        let sfc = gen_sfc_from_widget_src_shadcn(r#"
widget App {
    model { var show bool = false }
    view {
        dialog(open: .show) {
            text "hi"
        }
    }
}
"#);
        assert!(
            sfc.contains("<Dialog v-model:open=\"show\""),
            "dialog → <Dialog v-model:open>:\n{}",
            sfc
        );
    }

    /// Item 1 regression: builtin `alertdialog` keeps v-model:open (this was
    /// the original silent-drop: extract_state_ref only matched bare Ident).
    #[test]
    fn test_alertdialog_vmodel_open_real_dsl() {
        let sfc = gen_sfc_from_widget_src_shadcn(r#"
widget App {
    model { var confirm_open bool = false }
    view {
        alertdialog(open: .confirm_open) {
            alertdialogtitle(text: "Sure?")
        }
    }
}
"#);
        assert!(
            sfc.contains("<AlertDialog v-model:open=\"confirm_open\""),
            "alertdialog → <AlertDialog v-model:open>:\n{}",
            sfc
        );
    }

    #[test]
    fn test_generate_shadcn_attrs_tooltip() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tooltip with content and side
        props.insert("content".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("Help text".into())));
        props.insert("side".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("right".into())));
        let (attrs, slot_content, _) = gen.generate_shadcn_attrs("tooltip", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("side=\"right\"")));
        assert!(slot_content.is_some());
        assert_eq!(slot_content.unwrap(), "Help text");
    }

    #[test]
    fn test_generate_shadcn_attrs_spinner() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test spinner/skeleton
        props.insert("class".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("w-10 h-10".into())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("spinner", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("class=\"w-10 h-10\"")));
    }

    #[test]
    fn test_shadcn_registry_phase4_components() {
        let registry = ShadcnRegistry::new();

        // Check Phase 4 component mappings
        assert!(registry.has_component("modal"));
        assert!(registry.has_component("tooltip"));
        assert!(registry.has_component("spinner"));

        // Check primary component names
        assert_eq!(registry.primary_component("modal"), Some("Dialog"));
        assert_eq!(registry.primary_component("tooltip"), Some("Tooltip"));
        assert_eq!(registry.primary_component("spinner"), Some("Skeleton"));
    }

    // ========================================
    // Phase 5: Data Components Tests
    // ========================================

    #[test]
    fn test_generate_shadcn_attrs_table() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test table
        props.insert("class".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("w-full".into())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("table", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("class=\"w-full\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_table_cells() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test th with colspan
        props.insert("colspan".to_string(), AuraPropValue::Expr(crate::ast::Expr::Int(2)));
        let (attrs, _, _) = gen.generate_shadcn_attrs("th", &props, &events);

        assert!(attrs.iter().any(|a| a.contains(":colspan=\"2\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_tree() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tree
        props.insert("class".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("pl-4".into())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("tree", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("class=\"pl-4\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_tree_item() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tree_item with text
        props.insert("text".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("Node 1".into())));
        let (attrs, slot_content, _) = gen.generate_shadcn_attrs("tree_item", &props, &events);

        assert!(slot_content.is_some());
        assert_eq!(slot_content.unwrap(), "Node 1");
    }

    #[test]
    fn test_shadcn_registry_phase5_components() {
        let registry = ShadcnRegistry::new();

        // Check Phase 5 component mappings
        assert!(registry.has_component("table"));
        assert!(registry.has_component("thead"));
        assert!(registry.has_component("tbody"));
        assert!(registry.has_component("tr"));
        assert!(registry.has_component("th"));
        assert!(registry.has_component("td"));
        assert!(registry.has_component("avatar"));

        // Check primary component names
        assert_eq!(registry.primary_component("table"), Some("Table"));
        assert_eq!(registry.primary_component("thead"), Some("TableHeader"));
        assert_eq!(registry.primary_component("tbody"), Some("TableBody"));
        assert_eq!(registry.primary_component("tr"), Some("TableRow"));
        assert_eq!(registry.primary_component("th"), Some("TableHead"));
        assert_eq!(registry.primary_component("td"), Some("TableCell"));
        assert_eq!(registry.primary_component("avatar"), Some("Avatar"));
    }

    // ========================================
    // Phase 6: Form Components Tests
    // ========================================

    #[test]
    fn test_generate_shadcn_attrs_radiogroup() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test radiogroup with v-model
        props.insert("value".to_string(), AuraPropValue::Expr(crate::ast::Expr::Ident("selectedOption".into())));
        props.insert("name".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("options".into())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("radiogroup", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("v-model=\"selectedOption\"")));
        assert!(attrs.iter().any(|a| a.contains("name=\"options\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_radio() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test radio with value and label
        props.insert("value".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("option1".into())));
        props.insert("label".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("Option 1".into())));
        let (attrs, slot_content, _) = gen.generate_shadcn_attrs("radio", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("value=\"option1\"")));
        assert!(slot_content.is_some());
        assert_eq!(slot_content.unwrap(), "Option 1");
    }

    #[test]
    fn test_generate_shadcn_attrs_radio_disabled() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test disabled radio
        props.insert("value".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("option2".into())));
        props.insert("disabled".to_string(), AuraPropValue::Expr(crate::ast::Expr::Bool(true)));
        let (attrs, _, _) = gen.generate_shadcn_attrs("radio", &props, &events);

        assert!(attrs.iter().any(|a| a == "disabled"));
    }

    #[test]
    fn test_shadcn_registry_phase6_components() {
        let registry = ShadcnRegistry::new();

        // Check Phase 6 component mappings
        assert!(registry.has_component("slider"));
        assert!(registry.has_component("radio"));
        assert!(registry.has_component("radiogroup"));

        // Check primary component names
        assert_eq!(registry.primary_component("slider"), Some("Slider"));
        // radio maps to RadioGroup with RadioGroupItem as secondary
        assert_eq!(registry.primary_component("radio"), Some("RadioGroup"));
        assert_eq!(registry.primary_component("radiogroup"), Some("RadioGroup"));

        // Verify both RadioGroup and RadioGroupItem are in the component list
        let (_, components) = registry.get("radio").unwrap();
        assert!(components.contains(&"RadioGroup"));
        assert!(components.contains(&"RadioGroupItem"));
    }

    // ========================================================================
    // Router Tests (Plan 105)
    // ========================================================================

    #[test]
    fn test_router_generation() {
        use crate::aura::AuraRoute;

        let routes = vec![
            AuraRoute {
                path: "/".to_string(),
                module: "index".to_string(),
                widget_name: "Index".to_string(),
                params: vec![],
            },
            AuraRoute {
                path: "/about".to_string(),
                module: "about".to_string(),
                widget_name: "About".to_string(),
                params: vec![],
            },
            AuraRoute {
                path: "/user/:id".to_string(),
                module: "user".to_string(),
                widget_name: "User".to_string(),
                params: vec!["id".to_string()],
            },
        ];

        let output = VueGenerator::generate_router_file(&routes);

        // Check imports
        assert!(output.contains("import { createRouter, createWebHashHistory }"));

        // Check lazy loading imports (Plan 106)
        assert!(output.contains("component: () => import('@/pages/index.vue')"));
        assert!(output.contains("component: () => import('@/pages/about.vue')"));
        assert!(output.contains("component: () => import('@/pages/user.vue')"));

        // Check route definitions
        assert!(output.contains("path: '/'"));
        assert!(output.contains("path: '/about'"));
        assert!(output.contains("path: '/user/:id'"));

        // Check route with params has props: true
        assert!(output.contains("props: true"));
    }

    #[test]
    fn test_router_generation_empty() {
        let routes: Vec<crate::aura::AuraRoute> = vec![];
        let output = VueGenerator::generate_router_file(&routes);

        // Should still generate valid router structure
        assert!(output.contains("import { createRouter, createWebHashHistory }"));
        assert!(output.contains("const routes: RouteRecordRaw[] = ["));
        assert!(output.contains("export default router"));
    }

    #[test]
    fn test_button_with_text_full_widget() {
        use crate::aura::AuraWidget;
        use crate::aura::AuraNode;
        use std::collections::HashMap;

        // Create a simple Button element node
        let button_node = AuraNode::Element {
            tag: "Button".to_string(),
            props: {
                let mut map = HashMap::new();
                map.insert("text".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("Click Me".into())));
                map.insert("style".to_string(), AuraPropValue::Expr(crate::ast::Expr::Str("px-4 py-2 bg-blue-500".into())));
                map
            },
            events: HashMap::new(),
            children: vec![],
            span: None,
            debug_id: None,
        };

        let widget = AuraWidget {
            name: "Test".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![],
            view_tree: button_node,
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
        };

        let mut gen = VueGenerator::new_shadcn();
        let vue_code = gen.generate(&widget).unwrap();

        // Check that the button is NOT self-closing and has text content
        assert!(vue_code.contains("<Button") && vue_code.contains("Click Me"));
        // Should NOT be self-closing (should have >Click Me< pattern)
        assert!(vue_code.contains("Click Me") && vue_code.contains("</Button>"));
    }

    // ------------------------------------------------------------------
    // Widget `computed` regressions (AutoDown editor Phase 0 findings)
    // ------------------------------------------------------------------

    fn widget_with_computed(props: Vec<crate::aura::AuraProp>, computed: Vec<crate::aura::AuraComputed>) -> AuraWidget {
        AuraWidget {
            name: "Icon".to_string(),
            state_vars: vec![],
            computed,
            messages: vec![],
            view_tree: AuraNode::element("div"),
            handlers: HashMap::new(),
            props,
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
        }
    }

    fn str_prop(name: &str) -> crate::aura::AuraProp {
        crate::aura::AuraProp {
            name: name.to_string(),
            type_info: crate::ast::Type::StrSlice,
            default: None,
        }
    }

    /// Bug: `.language` (prop dot-ref) in a computed block was generated as
    /// `self.language`. It must resolve to `props.language`.
    #[test]
    fn test_computed_prop_dot_ref_uses_props_not_self() {
        use crate::ast::Expr;
        let widget = widget_with_computed(
            vec![str_prop("language")],
            vec![crate::aura::AuraComputed {
                name: "label".to_string(),
                expr: Expr::Dot(Box::new(Expr::Ident("self".into())), "language".into()),
            }],
        );

        let mut gen = VueGenerator::new();
        let sfc = gen.generate(&widget).unwrap();

        assert!(
            sfc.contains("const label = computed<string>(() => props.language)"),
            "computed prop must use props.language:\n{}",
            sfc
        );
        assert!(!sfc.contains("self.language"), "no self. in output:\n{}", sfc);
    }

    /// Bug: string concatenation in a computed block was inferred as
    /// `computed<number>`. `"data:" + .language` must be `computed<string>`.
    #[test]
    fn test_computed_string_concat_infers_string() {
        use crate::ast::Expr;
        use auto_val::Op;
        let widget = widget_with_computed(
            vec![str_prop("language")],
            vec![crate::aura::AuraComputed {
                name: "full".to_string(),
                expr: Expr::Bina(
                    Box::new(Expr::Str("data:".into())),
                    Op::Add,
                    Box::new(Expr::Dot(Box::new(Expr::Ident("self".into())), "language".into())),
                ),
            }],
        );

        let mut gen = VueGenerator::new();
        let sfc = gen.generate(&widget).unwrap();

        assert!(
            sfc.contains("const full = computed<string>(() => 'data:' + props.language)"),
            "string concat must infer string:\n{}",
            sfc
        );
    }

    /// Bug: bare identifiers (`lang => language`) and plain function calls
    /// (`btoa(language)`) in a computed block failed to parse with
    /// "Expected term, got RBrace". They must parse and resolve against the
    /// widget's props.
    #[test]
    fn test_computed_parses_bare_ident_and_plain_call() {
        let src = r#"
widget Icon(language: str) {
    computed {
        label => language
        encoded => btoa(language)
    }
    view {
        div {
            class: "icon",
            "hi"
        }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::parser::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("widget computed with bare ident/call must parse");

        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");

        let mut gen = VueGenerator::new();
        let sfc = gen.generate(&widget).unwrap();

        assert!(
            sfc.contains("const label = computed<string>(() => props.language)"),
            "bare ident resolves to prop:\n{}",
            sfc
        );
        assert!(
            sfc.contains("btoa(props.language)"),
            "plain call passes through with resolved arg:\n{}",
            sfc
        );
    }

    // ====================================================================
    // Generic DOM events: keyboard/mouse/wheel events, event modifiers,
    // the $event object, and window/document-level listeners.
    // ====================================================================

    /// Parse a widget source and generate its Vue SFC (full pipeline).
    fn gen_sfc_from_widget_src(src: &str) -> String {
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::parser::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("widget source must parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
        let mut gen = VueGenerator::new();
        gen.generate(&widget).expect("generate SFC")
    }

    /// Same as gen_sfc_from_widget_src, but in shadcn-vue mode (real widgets
    /// compile with `render: "vue"` + shadcn enabled; dialog/input arms only
    /// run in this mode).
    fn gen_sfc_from_widget_src_shadcn(src: &str) -> String {
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::parser::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("widget source must parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
        let mut gen = VueGenerator::new_shadcn();
        gen.generate(&widget).expect("generate SFC")
    }

    // ====================================================================
    // v-model contracts (Item 2): a widget can declare a true Vue v-model
    // target by pairing a `modelValue` prop with a quoted msg variant whose
    // name is a literal Vue emit name (`msg Msg { "update:modelValue"(str) }`).
    // ====================================================================

    /// Child side: quoted msg variant "update:modelValue" → typed defineEmits
    /// with a QUOTED key, a sanitized forwarding handler fn, and emit() with
    /// the verbatim event name.
    #[test]
    fn test_vmodel_contract_child_quoted_emit() {
        let sfc = gen_sfc_from_widget_src_shadcn(r#"
widget TextField(modelValue: str) {
    msg Msg { "update:modelValue"(str) }
    view {
        col {
            input { value: .modelValue, oninput: ."update:modelValue" }
        }
    }
    on {
        ."update:modelValue"(v) -> { }
    }
}
"#);
        assert!(
            sfc.contains("modelValue: string"),
            "modelValue prop declared:\n{}",
            sfc
        );
        assert!(
            sfc.contains("'update:modelValue': [string]"),
            "quoted emit key in defineEmits:\n{}",
            sfc
        );
        assert!(
            sfc.contains("emit('update:modelValue', v)"),
            "emit with verbatim name + payload:\n{}",
            sfc
        );
        assert!(
            sfc.contains("function update_modelValue(v: any): void"),
            "sanitized handler fn:\n{}",
            sfc
        );
        // Prop-backed value: one-way :modelValue down (never v-model on a prop),
        // @update:modelValue up — a controlled v-model contract.
        assert!(
            sfc.contains(":modelValue=\"modelValue\""),
            "one-way :modelValue binding:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@update:modelValue=\"update_modelValue\""),
            "update event wiring:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("v-model=\"modelValue\""),
            "no v-model on a read-only prop:\n{}",
            sfc
        );
    }

    /// Parent side (manual wiring, no sugar): a DSL parent can already bind a
    /// v-model contract via the modelValue prop + a quoted custom event key
    /// `on "update:modelValue":` (Plan-367 quoted event names).
    #[test]
    fn test_vmodel_contract_parent_manual_wiring() {
        let sfc = gen_sfc_from_widget_src(r#"
widget App {
    msg Msg { NameChanged(str) }
    model { var name str = "" }
    view {
        col {
            TextField(modelValue: .name) {
                on "update:modelValue": .NameChanged
            }
        }
    }
    on {
        .NameChanged(v) -> { .name = v }
    }
}
"#);
        assert!(
            sfc.contains(":modelValue=\"name\""),
            "modelValue prop down:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@update:modelValue=\"NameChanged\""),
            "quoted update event up:\n{}",
            sfc
        );
    }

    // ====================================================================
    // Dynamic components (dyn) and reactive watchers (watch).
    // ====================================================================

    /// dyn (.item.icon) { size: 16, class: "..." } inside a for loop →
    /// <component :is="(item.icon) as any" :size="16" class="..." />.
    #[test]
    fn test_dyn_component_in_for_loop() {
        let sfc = gen_sfc_from_widget_src(r#"
widget SlashMenu {
    model { var items list = [] }
    view {
        col {
            for item in .items {
                dyn (.item.icon) { size: 16, class: "w-4 h-4 shrink-0" }
            }
        }
    }
}
"#);
        assert!(
            sfc.contains(r#":is="(item.icon) as any""#),
            "dyn :is binding:\n{}",
            sfc
        );
        assert!(
            sfc.contains(r#"v-for="item in items""#),
            "v-for on dyn element:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("/ v-for"),
            "v-for not inserted after self-closing slash:\n{}",
            sfc
        );
        assert!(sfc.contains(r#":size="16""#), "extra prop bound:\n{}", sfc);
        assert!(
            sfc.contains(r#"class="w-4 h-4 shrink-0""#),
            "static class:\n{}",
            sfc
        );
    }

    /// dyn without parentheses: is as a plain prop; model-field source.
    #[test]
    fn test_dyn_component_is_prop_model_field() {
        let sfc = gen_sfc_from_widget_src(r#"
widget IconBox {
    model { var current_icon str = "x" }
    view {
        col {
            dyn { is: .current_icon }
        }
    }
}
"#);
        assert!(
            sfc.contains(r#"<component :is="(current_icon) as any" />"#),
            "is prop form:\n{}",
            sfc
        );
    }

    /// watch { .computed -> { ... } } → watch(filtered, () => { ... }) with
    /// on-handler body conventions (.field = state access via .value).
    #[test]
    fn test_watch_computed_source() {
        let sfc = gen_sfc_from_widget_src(r#"
widget SlashMenu {
    model {
        var query str = ""
        var selected_index int = 0
    }
    computed {
        filtered => query
    }
    view { col { text "hi" } }
    watch {
        .filtered -> { .selected_index = 0 }
    }
}
"#);
        assert!(
            sfc.contains("import { ref, computed, watch } from 'vue'"),
            "watch imported:\n{}",
            sfc
        );
        assert!(
            sfc.contains("watch(filtered, () => {"),
            "watch call:\n{}",
            sfc
        );
        assert!(
            sfc.contains("selected_index.value = 0"),
            "handler body transpiled like on block:\n{}",
            sfc
        );
    }

    /// Prop source needs a getter; .immediate/.deep become watch options;
    /// multiple sources become an array.
    #[test]
    fn test_watch_prop_source_and_modifiers() {
        let sfc = gen_sfc_from_widget_src(r#"
widget Scrollbar(ratio: int, viewport_h: int) {
    model { var thumb_h int = 0 }
    view { col { text "hi" } }
    watch {
        .ratio, .viewport_h.immediate -> { .thumb_h = .viewport_h }
        .ratio.deep -> { .thumb_h = 0 }
    }
}
"#);
        assert!(
            sfc.contains("watch([() => props.ratio, () => props.viewport_h], () => {"),
            "multi-source prop getters:\n{}",
            sfc
        );
        assert!(
            sfc.contains("{ immediate: true }"),
            "immediate option:\n{}",
            sfc
        );
        assert!(
            sfc.contains("{ deep: true }"),
            "deep option:\n{}",
            sfc
        );
        assert!(
            sfc.contains("thumb_h.value = props.viewport_h"),
            "prop read in body:\n{}",
            sfc
        );
    }

    // ====================================================================
    // defineExpose: widget-level `expose { ... }` block.
    // ====================================================================

    /// expose { .fit } where `fit` is an imported TS fn →
    /// defineExpose({ fit }) in <script setup>.
    #[test]
    fn test_expose_single_imported_fn() {
        let sfc = gen_sfc_from_widget_src(r#"
widget GraphView {
    use {
        fn: fitGraph from "src/front/utils/graph.ts"
    }
    model { var zoom int = 1 }
    view { col { text "hi" } }
    expose {
        .fitGraph
    }
}
"#);
        assert!(
            sfc.contains("defineExpose({ fitGraph })"),
            "defineExpose with imported fn:\n{}",
            sfc
        );
    }

    /// Multiple members on one line and across lines; an exposed `on`
    /// handler is emitted even though the template never references it.
    #[test]
    fn test_expose_multiple_members_and_handler() {
        let sfc = gen_sfc_from_widget_src(r#"
widget GraphView {
    msg Msg { Fit, Relayout }
    model { var zoom int = 1 }
    computed { doubled => .zoom * 2 }
    view { col { text "hi" } }
    on {
        .Fit -> { .zoom = 1 }
        .Relayout -> { .zoom = .zoom }
    }
    expose {
        .Fit, .Relayout
        .doubled
    }
}
"#);
        assert!(
            sfc.contains("defineExpose({ Fit, Relayout, doubled })"),
            "defineExpose with handler + computed names:\n{}",
            sfc
        );
        assert!(
            sfc.contains("function Fit()"),
            "exposed handler emitted despite no template use:\n{}",
            sfc
        );
        assert!(
            sfc.contains("function Relayout()"),
            "second exposed handler emitted:\n{}",
            sfc
        );
    }

    /// Expose a model state var and a template ref (ref: "graphEl" in the
    /// view): both exist as script-setup refs; defineExpose exposes the ref
    /// objects (Vue's expose proxy unwraps them on parent access).
    #[test]
    fn test_expose_state_and_template_ref() {
        let sfc = gen_sfc_from_widget_src(r#"
widget Editor {
    model { var content str = "" }
    view {
        col {
            ref: "rootEl"
            text "editor"
        }
    }
    expose {
        .content, .rootEl
    }
}
"#);
        assert!(
            sfc.contains("const content = ref"),
            "state ref exists:\n{}",
            sfc
        );
        assert!(
            sfc.contains("const rootEl = ref<HTMLElement | null>(null)"),
            "template ref exists:\n{}",
            sfc
        );
        assert!(
            sfc.contains("defineExpose({ content, rootEl })"),
            "defineExpose with state + template ref:\n{}",
            sfc
        );
    }

    /// A widget without an expose block emits no defineExpose.
    #[test]
    fn test_no_expose_block_unchanged() {
        let sfc = gen_sfc_from_widget_src(r#"
widget Plain {
    model { var count int = 0 }
    view { col { text "hi" } }
}
"#);
        assert!(
            !sfc.contains("defineExpose"),
            "no defineExpose without expose block:\n{}",
            sfc
        );
    }

    /// Parent side: `ref: "canvasRef"` on a child component → static
    /// `ref="canvasRef"` attribute + `ref<any>` declaration, so handlers can
    /// call the child's exposed methods via `.canvasRef.method()`.
    #[test]
    fn test_component_template_ref() {
        let sfc = gen_sfc_from_widget_src(r#"
widget App {
    msg Msg { DoFit }
    model { var n int = 0 }
    view {
        col {
            GraphView { ref: "canvasRef" }
            button "fit" { onclick: .DoFit }
        }
    }
    on {
        .DoFit -> { .canvasRef.Fit() }
    }
}
"#);
        assert!(
            sfc.contains(r#"<GraphView ref="canvasRef""#),
            "static ref attr on child component:\n{}",
            sfc
        );
        assert!(
            sfc.contains("const canvasRef = ref<any>(null)"),
            "component ref typed any:\n{}",
            sfc
        );
        assert!(
            sfc.contains("canvasRef.value!.Fit()"),
            "exposed method call through the ref:\n{}",
            sfc
        );
    }

    /// Element-level generic events: keyboard, mouse, wheel, contextmenu.
    #[test]
    fn test_generic_dom_events() {
        let sfc = gen_sfc_from_widget_src(r#"
widget Menu {
    msg Msg { Key, Down, Move, Up, Wheel, Ctx }
    model { var x int = 0 }
    view {
        col {
            onkeydown: .Key,
            onmousedown: .Down,
            onmousemove: .Move,
            onmouseup: .Up,
            onwheel: .Wheel,
            oncontextmenu: .Ctx
        }
    }
    on {
        .Key -> { .x = 1 }
        .Down -> { .x = 2 }
        .Move -> { .x = 3 }
        .Up -> { .x = 4 }
        .Wheel -> { .x = 5 }
        .Ctx -> { .x = 6 }
    }
}
"#);
        assert!(sfc.contains("@keydown=\"Key\""), "@keydown emitted:\n{}", sfc);
        assert!(sfc.contains("@mousedown=\"Down\""), "@mousedown emitted:\n{}", sfc);
        assert!(sfc.contains("@mousemove=\"Move\""), "@mousemove emitted:\n{}", sfc);
        assert!(sfc.contains("@mouseup=\"Up\""), "@mouseup emitted:\n{}", sfc);
        assert!(sfc.contains("@wheel=\"Wheel\""), "@wheel emitted:\n{}", sfc);
        assert!(sfc.contains("@contextmenu=\"Ctx\""), "@contextmenu emitted:\n{}", sfc);
    }

    /// Event modifiers: key modifiers (up/escape), prevent, stop.
    #[test]
    fn test_event_modifiers() {
        let sfc = gen_sfc_from_widget_src(r#"
widget Menu {
    msg Msg { MoveUp, Close, Ctx, Tap }
    model { var x int = 0 }
    view {
        col {
            onkeydown.up: .MoveUp,
            onkeydown.escape.prevent: .Close,
            oncontextmenu.prevent: .Ctx,
            onclick.stop: .Tap
        }
    }
    on {
        .MoveUp -> { .x = 1 }
        .Close -> { .x = 2 }
        .Ctx -> { .x = 3 }
        .Tap -> { .x = 4 }
    }
}
"#);
        assert!(sfc.contains("@keydown.up=\"MoveUp\""), "key modifier up:\n{}", sfc);
        assert!(
            sfc.contains("@keydown.esc.prevent=\"Close\""),
            "escape normalizes to esc, prevent appended:\n{}",
            sfc
        );
        assert!(sfc.contains("@contextmenu.prevent=\"Ctx\""), "prevent modifier:\n{}", sfc);
        assert!(sfc.contains("@click.stop=\"Tap\""), "stop modifier:\n{}", sfc);
    }

    /// The $event object flows into the template handler call, with field
    /// access ($event.key, $event.clientY) preserved.
    #[test]
    fn test_event_object_param() {
        let sfc = gen_sfc_from_widget_src(r#"
widget Menu {
    msg Msg { Key, Drag }
    model { var x int = 0 }
    view {
        col {
            onkeydown: .Key($event),
            onmousedown: .Drag($event.clientY)
        }
    }
    on {
        .Key(e) -> { .x = 1 }
        .Drag(y) -> { .x = 2 }
    }
}
"#);
        assert!(
            sfc.contains("@keydown=\"Key($event)\""),
            "$event passed to handler:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@mousedown=\"Drag($event.clientY)\""),
            "$event.clientY field access:\n{}",
            sfc
        );
        // Handler functions keep their declared params.
        assert!(sfc.contains("function Key(e: any)"), "Key param:\n{}", sfc);
        assert!(sfc.contains("function Drag(y: any)"), "Drag param:\n{}", sfc);
    }

    /// window-level mouse listeners: drag tracking outside the element.
    /// - with $event → wrapper function adapting args
    /// - without params → bare function reference
    #[test]
    fn test_global_window_mouse_listeners() {
        let sfc = gen_sfc_from_widget_src(r#"
widget Scrollbar {
    msg Msg { DragMove, DragEnd }
    model { var y int = 0 }
    view {
        col {
            onmousemove.window: .DragMove($event),
            onmouseup.window: .DragEnd
        }
    }
    on {
        .DragMove(e) -> { .y = 0 }
        .DragEnd -> { .y = 1 }
    }
}
"#);
        // No template attribute for global listeners.
        assert!(!sfc.contains("@mousemove"), "no @mousemove attr:\n{}", sfc);
        assert!(!sfc.contains("@mouseup"), "no @mouseup attr:\n{}", sfc);
        // add/remove pairs in onMounted/onUnmounted.
        assert!(
            sfc.contains("window.addEventListener('mousemove', __auto_gl_mousemove_DragMove)"),
            "window mousemove add:\n{}",
            sfc
        );
        assert!(
            sfc.contains("window.removeEventListener('mousemove', __auto_gl_mousemove_DragMove)"),
            "window mousemove remove:\n{}",
            sfc
        );
        assert!(
            sfc.contains("window.addEventListener('mouseup', DragEnd)"),
            "bare fn ref when no params:\n{}",
            sfc
        );
        assert!(
            sfc.contains("window.removeEventListener('mouseup', DragEnd)"),
            "bare fn ref removal:\n{}",
            sfc
        );
        // Wrapper adapts $event → e.
        assert!(
            sfc.contains("function __auto_gl_mousemove_DragMove(e: any) {\n  DragMove(e)\n}"),
            "wrapper adapts event arg:\n{}",
            sfc
        );
        // Lifecycle imports present even without an explicit .Init/.Destroy.
        assert!(
            sfc.contains("onMounted") && sfc.contains("onUnmounted"),
            "lifecycle hooks imported/emitted:\n{}",
            sfc
        );
    }

    /// document-level wheel lock: capture phase + preventDefault, with
    /// passive: false (required by Chrome for document-level wheel listeners).
    #[test]
    fn test_global_document_wheel_capture_prevent() {
        let sfc = gen_sfc_from_widget_src(r#"
widget CodeMenu {
    msg Msg { LockWheel }
    model { var locked int = 0 }
    view {
        col {
            onwheel.document.capture.prevent: .LockWheel($event)
        }
    }
    on {
        .LockWheel(e) -> { .locked = 1 }
    }
}
"#);
        assert!(
            sfc.contains(
                "document.addEventListener('wheel', __auto_gl_wheel_LockWheel, { capture: true, passive: false })"
            ),
            "capture+passive options on add:\n{}",
            sfc
        );
        assert!(
            sfc.contains(
                "document.removeEventListener('wheel', __auto_gl_wheel_LockWheel, { capture: true })"
            ),
            "capture option on remove (no passive):\n{}",
            sfc
        );
        assert!(
            sfc.contains("function __auto_gl_wheel_LockWheel(e: any) {\n  e.preventDefault()\n  LockWheel(e)\n}"),
            "wrapper calls preventDefault then handler:\n{}",
            sfc
        );
    }

    // ====================================================================
    // Widget `use { ... }` external TS/Vue imports (escape hatch)
    // ====================================================================

    /// The widget-level `use { ... }` block parses into `decl.ext_imports`
    /// with kind, symbols, and path intact.
    #[test]
    fn test_ext_use_block_parses() {
        let src = r#"
widget Icon(language: str) {
    use {
        fn: getLanguageIconUrl from "src/front/utils/codeBlockLanguage.ts"
        fn: marked, purify from "marked"
        component: FancyBadge from "src/front/components/FancyBadge.vue"
        component: Smile from "lucide-vue-next"
        composable: useClock from "src/front/composables/useClock.ts"
    }
    view {
        div { "hi" }
    }
}
"#;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::parser::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("widget use block must parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");

        assert_eq!(decl.ext_imports.len(), 5, "all five entries parse");
        assert_eq!(decl.ext_imports[0].kind, crate::ast::ExtImportKind::Fn);
        assert_eq!(decl.ext_imports[0].symbols.len(), 1);
        assert_eq!(decl.ext_imports[0].symbols[0].as_str(), "getLanguageIconUrl");
        assert_eq!(decl.ext_imports[0].path.as_str(), "src/front/utils/codeBlockLanguage.ts");
        // Comma-separated symbol list
        assert_eq!(decl.ext_imports[1].symbols.len(), 2);
        assert_eq!(decl.ext_imports[1].symbols[1].as_str(), "purify");
        assert_eq!(decl.ext_imports[2].kind, crate::ast::ExtImportKind::Component);
        assert_eq!(decl.ext_imports[4].kind, crate::ast::ExtImportKind::Composable);
    }

    /// `fn:` imports become named ES imports; npm specifiers pass through,
    /// local files are rewritten to the `@/ext/...` alias. The imported
    /// symbol is callable from `on` handlers and `computed`.
    #[test]
    fn test_ext_fn_import_and_call_sites() {
        let sfc = gen_sfc_from_widget_src(r#"
widget Icon(language: str) {
    use {
        fn: getLanguageIconUrl from "src/front/utils/codeBlockLanguage.ts"
        fn: marked from "marked"
    }
    msg Msg { Render }
    model { var url str = "" }
    computed {
        iconUrl => getLanguageIconUrl(language)
    }
    view {
        col {
            onclick: .Render
        }
    }
    on {
        .Render -> { .url = getLanguageIconUrl(.language) }
    }
}
"#);
        // Local path → @/ext alias, .ts extension dropped.
        assert!(
            sfc.contains("import { getLanguageIconUrl } from '@/ext/src/front/utils/codeBlockLanguage'"),
            "local fn import via @/ext alias:\n{}",
            sfc
        );
        // npm specifier passes through unchanged.
        assert!(
            sfc.contains("import { marked } from 'marked'"),
            "npm fn import passthrough:\n{}",
            sfc
        );
        // Callable from computed (prop resolves to props.language).
        assert!(
            sfc.contains("getLanguageIconUrl(props.language)"),
            "fn call in computed:\n{}",
            sfc
        );
        // Callable from on handler.
        assert!(
            sfc.contains("url.value = getLanguageIconUrl(props.language)"),
            "fn call in on handler:\n{}",
            sfc
        );
    }

    /// `component:` from a local `.vue` file → default import; usable as a
    /// view tag (PascalCase or snake_case) with generic `:prop` bindings
    /// and `@event` listeners.
    #[test]
    fn test_ext_component_local_vue() {
        let sfc = gen_sfc_from_widget_src(r#"
widget App {
    use {
        component: FancyBadge from "src/front/components/FancyBadge.vue"
    }
    msg Msg { Picked }
    model { var label str = "hi" }
    view {
        col {
            FancyBadge {
                label: .label,
                onselected: .Picked
            }
            fancy_badge {
                label: .label
            }
        }
    }
    on {
        .Picked -> { .label = "picked" }
    }
}
"#);
        assert!(
            sfc.contains("import FancyBadge from '@/ext/src/front/components/FancyBadge.vue'"),
            "default import for local .vue component:\n{}",
            sfc
        );
        // Both tag spellings render as the PascalCase component.
        let count = sfc.matches("<FancyBadge").count();
        assert_eq!(count, 2, "both tag spellings instantiate FancyBadge:\n{}", sfc);
        // Generic prop binding + event listener.
        assert!(sfc.contains(":label=\"label\""), "prop v-bind:\n{}", sfc);
        assert!(sfc.contains("@selected=\"Picked\""), "event listener:\n{}", sfc);
        // No fallback `@/components/FancyBadge.vue` import (registry path).
        assert!(
            !sfc.contains("from '@/components/FancyBadge.vue'"),
            "ext component must not use the @/components fallback:\n{}",
            sfc
        );
    }

    /// `component:` from an npm package → named import (lucide-style).
    #[test]
    fn test_ext_component_npm_named_import() {
        let sfc = gen_sfc_from_widget_src(r#"
widget App {
    use {
        component: Smile from "lucide-vue-next"
    }
    view {
        col {
            Smile { }
        }
    }
}
"#);
        assert!(
            sfc.contains("import { Smile } from 'lucide-vue-next'"),
            "named npm component import:\n{}",
            sfc
        );
        assert!(sfc.contains("<Smile"), "component tag rendered:\n{}", sfc);
    }

    /// `composable:` → named import + a single call at `<script setup>`
    /// top level, return value bound to a derived local const.
    #[test]
    fn test_ext_composable_setup_call() {
        let sfc = gen_sfc_from_widget_src(r#"
widget App {
    use {
        composable: useMenuBounds from "src/front/composables/useMenuBounds.ts"
    }
    msg Msg { Open }
    model { var x int = 0 }
    view {
        col {
            onclick: .Open
        }
    }
    on {
        .Open -> { .x = 1 }
    }
}
"#);
        assert!(
            sfc.contains("import { useMenuBounds } from '@/ext/src/front/composables/useMenuBounds'"),
            "composable import:\n{}",
            sfc
        );
        assert!(
            sfc.contains("const menuBounds = useMenuBounds()"),
            "composable called at setup top level:\n{}",
            sfc
        );
    }

    /// The new mechanism expresses the AutoDownEditor case: a declared
    /// `component: AutoDownEditor from "@autodown/editor"` produces the
    /// same import + PascalCase tag as the hardcoded registry path.
    #[test]
    fn test_ext_component_expresses_autodown_editor_case() {
        let sfc = gen_sfc_from_widget_src(r#"
widget App {
    use {
        component: AutoDownEditor from "@autodown/editor"
    }
    model { var body str = "" }
    view {
        col {
            AutoDownEditor {
                content: .body
            }
        }
    }
}
"#);
        assert!(
            sfc.contains("import { AutoDownEditor } from '@autodown/editor'"),
            "same import shape as the registry path:\n{}",
            sfc
        );
        assert!(sfc.contains("<AutoDownEditor"), "component tag:\n{}", sfc);
        assert!(sfc.contains(":content=\"body\""), "prop binding:\n{}", sfc);
    }

    // ====================================================================
    // Quoted custom event names: on "autodown:slash-open" — global
    // (document/window) listeners and element-level bindings.
    // ====================================================================

    /// document-level CustomEvent listeners with ':'/'-' in the event name
    /// (SlashMenu integration: a hand-written TS extension dispatches
    /// `autodown:slash-open` etc. on document).
    #[test]
    fn test_global_custom_event_listeners() {
        let sfc = gen_sfc_from_widget_src(r#"
widget SlashMenu {
    msg Msg { OnOpen, OnClose }
    model { var query str = "" }
    view {
        col {
            on "autodown:slash-open".document: .OnOpen($event),
            on "autodown:slash-close".document: .OnClose
        }
    }
    on {
        .OnOpen(e) -> { .query = "x" }
        .OnClose -> { .query = "" }
    }
}
"#);
        // No template attribute for global listeners.
        assert!(!sfc.contains("@autodown:slash-open"), "no template attr:\n{}", sfc);
        // add/remove pairs carry the raw custom event name.
        assert!(
            sfc.contains("document.addEventListener('autodown:slash-open', __auto_gl_autodown_slash_open_OnOpen)"),
            "custom event add (sanitized wrapper name):\n{}",
            sfc
        );
        assert!(
            sfc.contains("document.removeEventListener('autodown:slash-open', __auto_gl_autodown_slash_open_OnOpen)"),
            "custom event remove:\n{}",
            sfc
        );
        // No-param handler → bare function reference.
        assert!(
            sfc.contains("document.addEventListener('autodown:slash-close', OnClose)"),
            "bare fn ref:\n{}",
            sfc
        );
        // Wrapper adapts the DOM event arg.
        assert!(
            sfc.contains("function __auto_gl_autodown_slash_open_OnOpen(e: any) {\n  OnOpen(e)\n}"),
            "wrapper body:\n{}",
            sfc
        );
    }

    /// Element-level custom event binding (child component emit with a
    /// ':'/'-' name). Verified against @vue/compiler-dom + runtime-dom:
    /// `@autodown:slash-open` compiles and patches to
    /// addEventListener('autodown:slash-open').
    #[test]
    fn test_element_custom_event_listener() {
        let sfc = gen_sfc_from_widget_src(r#"
widget Menu {
    msg Msg { Poked }
    model { var x int = 0 }
    view {
        col {
            on "demo:poke": .Poked($event)
        }
    }
    on {
        .Poked(e) -> { .x = 1 }
    }
}
"#);
        assert!(
            sfc.contains("@demo:poke=\"Poked($event)\""),
            "element-level custom event binding:\n{}",
            sfc
        );
    }

    // ====================================================================
    // style_obj: inline-style object binding (:style), distinct from the
    // style: { class: cond } dynamic-class binding.
    // ====================================================================

    /// style_obj generates a Vue :style object binding; values are arbitrary
    /// expressions including f-string px concatenation; hyphenated CSS
    /// property names are quoted.
    #[test]
    fn test_style_obj_inline_style_binding() {
        let sfc = gen_sfc_from_widget_src(r#"
widget Popover {
    msg Msg { Nop }
    model {
        var menu_x int = 50
        var menu_y int = 100
        var menu_vis str = "visible"
    }
    view {
        col {
            style_obj: { top: f"${.menu_y}px", left: f"${.menu_x}px", "z-index": 50, visibility: .menu_vis }
        }
    }
    on {
        .Nop -> { .menu_x = 0 }
    }
}
"#);
        assert!(
            sfc.contains(":style=\"({ top: `${menu_y}px`, left: `${menu_x}px`, 'z-index': 50, visibility: menu_vis } as any)\""),
            ":style object binding:\n{}",
            sfc
        );
        // Must NOT be emitted as a class binding.
        assert!(!sfc.contains(":class=\"{ top:"), "not a class binding:\n{}", sfc);
    }

    /// The classic style: { class: cond } map stays a dynamic class binding;
    /// hyphenated class names are quoted (previously emitted as a bare,
    /// invalid JS key).
    #[test]
    fn test_style_class_binding_quotes_hyphenated_keys() {
        let sfc = gen_sfc_from_widget_src(r#"
widget Todo {
    msg Msg { Nop }
    model { var done int = 0 }
    view {
        col {
            text "x" { style: { completed: .done, "line-through": .done } }
        }
    }
    on {
        .Nop -> { .done = 1 }
    }
}
"#);
        assert!(
            sfc.contains(":class=\"{ completed: done, 'line-through': done }\""),
            "hyphenated class key quoted:\n{}",
            sfc
        );
    }

    // ====================================================================
    // Slots: `slot` outlet in a widget view (default + named) and
    // `slot(name:) { ... }` named-slot targeting at the parent side.
    // ====================================================================

    /// Bare `slot` in a widget view → default outlet `<slot />`
    /// (previously swallowed by the unknown-tag div fallback).
    #[test]
    fn test_slot_outlet_default() {
        let sfc = gen_sfc_from_widget_src(r#"
widget Panel {
    model { var n int = 0 }
    view {
        col {
            class: "panel",
            slot
        }
    }
}
"#);
        assert!(sfc.contains("<slot />"), "default slot outlet:\n{}", sfc);
        assert!(
            !sfc.contains("<div>\n</div>") && !sfc.contains("<div />"),
            "slot must not fall back to an empty div:\n{}",
            sfc
        );
    }

    /// `slot(name: "header")` in a widget view → named outlet
    /// `<slot name="header" />`.
    #[test]
    fn test_slot_outlet_named() {
        let sfc = gen_sfc_from_widget_src(r#"
widget Panel {
    model { var n int = 0 }
    view {
        col {
            slot(name: "header")
            slot
        }
    }
}
"#);
        assert!(
            sfc.contains("<slot name=\"header\" />"),
            "named slot outlet:\n{}",
            sfc
        );
        assert!(sfc.contains("<slot />"), "default slot outlet too:\n{}", sfc);
    }

    /// Parent side: `slot(name: "header") { ... }` inside a component
    /// instantiation's children block → `<template #header>...</template>`.
    #[test]
    fn test_slot_named_template_parent_side() {
        let sfc = gen_sfc_from_widget_src(r#"
widget App {
    model { var n int = 0 }
    view {
        col {
            Panel(title: "hi") {
                slot(name: "header") {
                    text "My Title"
                }
                text "body content"
            }
        }
    }
}
"#);
        assert!(
            sfc.contains("<template #header>"),
            "named slot template in parent:\n{}",
            sfc
        );
        assert!(
            sfc.contains(">My Title</span>"),
            "named slot content rendered:\n{}",
            sfc
        );
        assert!(
            sfc.contains(">body content</span>"),
            "default-slot child still rendered:\n{}",
            sfc
        );
    }

    /// Regression: plain children passed to a component still emit
    /// unchanged (no <template> wrapping, no slot outlet interference).
    #[test]
    fn test_slot_default_children_unchanged() {
        let sfc = gen_sfc_from_widget_src(r#"
widget App {
    model { var n int = 0 }
    view {
        col {
            Panel(title: "hi") {
                text "plain child"
            }
        }
    }
}
"#);
        assert!(
            sfc.contains(">plain child</span>"),
            "default children unchanged:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("<template #"),
            "no named-slot template without slot(name:):\n{}",
            sfc
        );
    }

    /// Outlet-less sub-widget receiving children → build-time warning via
    /// AuraWidget::slot_children_warnings; widget WITH a matching outlet
    /// produces no warning.
    #[test]
    fn test_slot_children_warning() {
        let parse_widget = |src: &str| {
            let session = crate::session::CompilerSession::ui();
            let mut parser = crate::parser::Parser::from(src).with_session(session);
            let ast = parser.parse().expect("widget source must parse");
            let decl = ast
                .stmts
                .iter()
                .find_map(|s| match s {
                    crate::ast::Stmt::WidgetDecl(d) => Some(d),
                    _ => None,
                })
                .expect("widget decl");
            crate::aura::extract_widget_from_decl(decl).expect("extract widget")
        };

        let outletless = parse_widget(r#"
widget Panel {
    model { var n int = 0 }
    view { col { text "no outlets" } }
}
"#);
        let with_outlets = parse_widget(r#"
widget Panel {
    model { var n int = 0 }
    view {
        col {
            slot(name: "header")
            slot
        }
    }
}
"#);
        assert!(with_outlets.slot_outlet_names().contains(&String::new()));
        assert!(with_outlets.slot_outlet_names().contains(&"header".to_string()));
        assert!(outletless.slot_outlet_names().is_empty());

        let app = parse_widget(r#"
widget App {
    model { var n int = 0 }
    view {
        col {
            Panel {
                slot(name: "header") {
                    text "My Title"
                }
                text "body content"
            }
        }
    }
}
"#);

        // Outlet-less target: both the named template and the default
        // children must warn.
        let mut outlets = std::collections::HashMap::new();
        outlets.insert("Panel".to_string(), outletless.slot_outlet_names());
        let warnings = app.slot_children_warnings(&outlets);
        assert_eq!(warnings.len(), 2, "named + default warnings:\n{:?}", warnings);
        assert!(warnings.iter().any(|w| w.contains("'header' slot outlet")));
        assert!(warnings.iter().any(|w| w.contains("no default slot outlet")));

        // Target with matching outlets: no warnings.
        outlets.insert("Panel".to_string(), with_outlets.slot_outlet_names());
        let warnings = app.slot_children_warnings(&outlets);
        assert!(warnings.is_empty(), "no warnings when outlets match:\n{:?}", warnings);
    }

    // ====================================================================
    // v-for :key — explicit `key:` prop override + index-var fallback.
    //
    // Syntax: any node in a loop body may take a `key: <expr>` prop; it is
    // emitted as `:key="<expr>"` and wins over the auto-generated reuse key.
    // ====================================================================

    /// Same as gen_sfc_from_widget_src, but registers sibling sub-widget
    /// names (the production app-build path passes them via
    /// with_sub_widgets; the known-sub-widget auto-:key logic only runs then).
    fn gen_sfc_with_sub_widgets(src: &str, subs: &[&str]) -> String {
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::parser::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("widget source must parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
        let mut gen = VueGenerator::new()
            .with_sub_widgets(subs.iter().map(|s| s.to_string()).collect());
        gen.generate(&widget).expect("generate SFC")
    }

    /// Explicit `key:` on a sub-widget component instantiation in a loop
    /// wins over the auto-key: exactly one :key, bound to the given expr.
    #[test]
    fn test_vfor_explicit_key_on_sub_widget() {
        let sfc = gen_sfc_with_sub_widgets(r#"
use tab: EditorTab

widget App {
    model { var tabs list = [] }
    view {
        col {
            for tab in .tabs {
                EditorTab(key: tab.path, path: tab.path)
            }
        }
    }
}
"#, &["EditorTab"]);
        assert!(
            sfc.contains(r#":key="tab.path""#),
            "explicit key emitted:\n{}",
            sfc
        );
        assert_eq!(
            sfc.matches(":key=").count(),
            1,
            "exactly one :key (no duplicate auto-key):\n{}",
            sfc
        );
        assert!(
            !sfc.contains("tab?.id"),
            "no auto-key ?.id chain when explicit key given:\n{}",
            sfc
        );
    }

    /// Explicit `key:` on a plain element in a loop is emitted as :key.
    #[test]
    fn test_vfor_explicit_key_on_plain_element() {
        let sfc = gen_sfc_from_widget_src(r#"
widget App {
    model { var names list = [] }
    view {
        col {
            for name in .names {
                span(key: name) { text "x" }
            }
        }
    }
}
"#);
        assert!(
            sfc.contains(r#"<span :key="name" v-for="name in names">"#),
            "explicit key on plain element:\n{}",
            sfc
        );
    }

    /// Regression: without an explicit key, the auto-key heuristic is
    /// unchanged (`'Tag-N-' + (item?.id ?? item)`).
    #[test]
    fn test_vfor_auto_key_unchanged_without_explicit_key() {
        let sfc = gen_sfc_with_sub_widgets(r#"
use tab: EditorTab

widget App {
    model { var tabs list = [] }
    view {
        col {
            for tab in .tabs {
                EditorTab(path: tab.path)
            }
        }
    }
}
"#, &["EditorTab"]);
        assert!(
            sfc.contains(r#":key="'EditorTab-1-' + (tab?.id ?? tab)""#),
            "auto-key heuristic unchanged:\n{}",
            sfc
        );
    }

    /// Indexed loop: the loop var is the primitive int index — the key must
    /// use the index itself, never `i?.id`.
    #[test]
    fn test_vfor_indexed_loop_key_uses_index() {
        // Sub-widget path (known sibling widget).
        let sfc = gen_sfc_with_sub_widgets(r#"
use tab: EditorTab

widget App {
    model { var tabs list = [] }
    view {
        col {
            for i, tab in .tabs {
                EditorTab(path: tab.path)
            }
        }
    }
}
"#, &["EditorTab"]);
        assert!(
            sfc.contains(r#":key="'EditorTab-1-' + i""#),
            "index var used as key:\n{}",
            sfc
        );
        assert!(!sfc.contains("i?.id"), "no ?.id on primitive index:\n{}", sfc);

        // Generic Vue-component path (Plan 360 fallback).
        let sfc2 = gen_sfc_from_widget_src(r#"
widget App {
    model { var tabs list = [] }
    view {
        col {
            for i, tab in .tabs {
                EditorTab(path: tab.path)
            }
        }
    }
}
"#);
        assert!(
            sfc2.contains(r#":key="'EditorTab-1-' + i""#),
            "index var used as key (generic path):\n{}",
            sfc2
        );
        assert!(!sfc2.contains("i?.id"), "no ?.id on primitive index (generic path):\n{}", sfc2);
    }

    /// Plan 043 M5 #3: an `if / else if / else if / else` chain must flatten
    /// into contiguous sibling `<template>` nodes (`v-if` → `v-else-if` →
    /// `v-else`), not nest as `<template v-else><template v-if>`.
    #[test]
    fn test_else_if_chain_flattens_to_v_else_if() {
        let sfc = gen_sfc_from_widget_src(r#"
widget Dispatch {
    model { var kind str = "Table" }
    view {
        if .kind == "Table" {
            text "T"
        } else if .kind == "Record" {
            text "R"
        } else if .kind == "Code" {
            text "C"
        } else {
            text "Other"
        }
    }
}
"#);
        // Head of the chain.
        assert!(
            sfc.contains(r#"<template v-if="kind == 'Table'">"#),
            "chain head v-if:\n{}",
            sfc
        );
        // Both continuations flatten to v-else-if (NOT nested v-else + v-if).
        assert!(
            sfc.contains(r#"<template v-else-if="kind == 'Record'">"#),
            "first continuation v-else-if:\n{}",
            sfc
        );
        assert!(
            sfc.contains(r#"<template v-else-if="kind == 'Code'">"#),
            "second continuation v-else-if:\n{}",
            sfc
        );
        // The bug symptom: a v-else immediately followed by a nested v-if.
        assert!(
            !sfc.contains("<template v-else>\n<template v-if="),
            "no nested v-else>v-if:\n{}",
            sfc
        );
        // Final else arm.
        assert!(
            sfc.contains("<template v-else>"),
            "trailing else arm:\n{}",
            sfc
        );
    }

    /// Plan 043 M5 #2: a multi-statement computed body must render its logic
    /// (not collapse to `undefined`). Previously expr_to_js had no Block branch
    /// so `x => { ...; return y }` emitted `computed(() => undefined)`.
    #[test]
    fn test_computed_multiline_body_renders_js() {
        let sfc = gen_sfc_from_widget_src(r#"
widget Counter {
    model { count int = 0 }
    computed {
        summary => {
            var label = "count="
            return label
        }
    }
    view { col { text "hi" } }
}
"#);
        // The computed body must keep its logic — the `return label` and the
        // local binding must survive, not be replaced by `undefined`.
        assert!(
            sfc.contains("const summary = computed"),
            "computed wrapper present:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("computed(() => undefined)"),
            "multiline body must not collapse to undefined:\n{}",
            sfc
        );
        assert!(
            sfc.contains("let label"),
            "local binding from block body survives:\n{}",
            sfc
        );
        assert!(
            sfc.contains("return label"),
            "return statement from block body survives:\n{}",
            sfc
        );
    }
}
