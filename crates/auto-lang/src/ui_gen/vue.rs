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
use crate::aura::{AuraBinOp, AuraEvent, AuraExpr, AuraNode, AuraPropValue, AuraStmt, AuraTextContent, AuraUnaryOp, AuraWidget, LogicPayload};
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

    /// Store dependencies from `use store:` (Plan 351)
    store_deps: Vec<String>,

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

    /// Handlers that need a loop-id parameter (e.g., "SelectNote" needs `i: any`)
    /// Populated during template generation, consumed during script generation.
    /// Maps handler name → loop variable name (e.g., "SelectNote" → "i").
    loop_param_handlers: HashMap<String, String>,

    /// Whether to generate handleChildDelete function (auto-wired when sub-widget emits Delete)
    needs_child_delete_handler: bool,

    /// Whether API functions were explicitly imported via `use back.api: ...`
    /// When true, skip AST scanning and use the explicit import list
    explicit_api_imports: bool,
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
            store_deps: Vec::new(),
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
            use_theme_toggle: false,
            use_curve_type: false,
            known_sub_widgets: HashSet::new(),
            current_loop_var: None,
            loop_param_handlers: HashMap::new(),
            needs_child_delete_handler: false,
            explicit_api_imports: false,
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
        self.handlers.clear();
        self.emit_events.clear();
        self.has_emit = false;
        self.component_refs.clear();
        self.lucide_icons.clear();
        self.wrapper_classes.clear();
        self.current_loop_var = None;
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
        self.use_theme_toggle = false;
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

        // Detect dark mode: check if widget has an isDark bool state variable
        self.has_dark_mode = widget.state_vars.iter().any(|s| s.name == "isDark");

        // Pre-populate state_names so expr_to_js recognizes refs during template generation
        for state in &widget.state_vars {
            self.state_names.push(state.name.clone());
        }

        // Register prop names (props are NOT refs — no .value suffix in script)
        for prop in &widget.props {
            self.prop_names.push(prop.name.clone());
        }

        // Plan 351: register 'store' as a pseudo-prop when store deps exist
        if !self.store_deps.is_empty() {
            self.prop_names.push("store".to_string());
        }

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

        // Plan 100: Add lang="ts" for TypeScript output
        let script_tag = if self.use_typescript {
            r#"<script setup lang="ts">"#
        } else {
            r#"<script setup>"#
        };

        Ok(format!(
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
"#,
            widget.name, script_tag, script, template, style
        ))
    }

    /// Generate <script setup> content
    fn generate_script(&mut self, widget: &AuraWidget) -> GenResult<String> {
        let mut script = String::new();

        // Determine needed imports
        let needs_ref = !widget.state_vars.is_empty();
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

        // Dark mode: detect system preference on mount
        if self.has_dark_mode {
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
            script.push_str("const props = defineProps<{\n");
            for prop in &widget.props {
                // Use 'any' for all prop types since Auto's type system
                // cannot fully map to TypeScript (e.g., object literals)
                if prop.default.is_some() {
                    script.push_str(&format!("  {}?: any\n", prop.name));
                } else {
                    script.push_str(&format!("  {}: any\n", prop.name));
                }
            }
            script.push_str("}>()\n\n");
        }

        // Generate emit if needed
        if self.has_emit {
            script.push_str("const emit = defineEmits<{\n");
            for event in &self.emit_events {
                script.push_str(&format!("  {}: []\n", event));
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
            // v1: single store → const store = useXxxStore()
            let first = &self.store_deps[0];
            script.push_str(&format!("const store = use{}Store()\n\n", first));
        }

        // Generate event handlers
        for (pattern, payload) in &widget.handlers {
            let handler_name = self.pattern_to_handler_name(pattern);
            let mut body = self.generate_handler_body(payload)?;
            // Auto-emit events for sub-widget handlers that match emit declarations
            if self.has_emit && self.emit_events.contains(&handler_name) {
                body.push_str(&format!("\nemit('{}')", handler_name));
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
            let pattern_key = format!(".{}", handler_name);
            let params_str = if let Some(loop_var) = self.loop_param_handlers.get(handler_name) {
                format!("{}: any", loop_var)
            } else {
                widget.handler_params.get(&pattern_key)
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
                script.push_str(&format!("{}function {}({}){} {{\n  {}\n}}\n\n", async_kw, handler_name, params_str, return_type, body));
            } else if handler_body.is_empty() {
                script.push_str(&format!("{}function {}({}){} {{\n  // TODO\n}}\n\n", async_kw, handler_name, params_str, return_type));
            } else {
                script.push_str(&format!("{}function {}({}){} {{\n  {}\n}}\n\n", async_kw, handler_name, params_str, return_type, handler_body));
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
            script.push_str("function handleChildDelete() {\n");
            // Find the deleted note by matching active_id, then remove from array
            script.push_str("  const idx = notes.value.findIndex((n: any) => n.id === notes.value[active_id.value]?.id)\n");
            script.push_str("  if (idx !== -1) notes.value.splice(idx, 1)\n");
            script.push_str("  if (notes.value.length > 0) {\n");
            script.push_str("    active_id.value = 0\n");
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
            script.push_str(&format!("onMounted({}() => {{\n  {}\n}})\n\n", async_kw, body));
        }
        // .Destroy → onUnmounted
        if let Some(destroy) = widget.lifecycle.iter().find(|l| l.name == "Destroy") {
            let body = self.generate_handler_body(&destroy.payload).unwrap_or_default();
            script.push_str(&format!("onUnmounted(() => {{\n  {}\n}})\n\n", body));
        }

        // Auto-enter edit mode for sub-widgets when receiving a new/empty item
        // If widget has editing state and a 'note' prop, auto-start editing when title is empty
        let has_editing = self.state_names.iter().any(|n| n == "editing");
        let has_note_prop = self.prop_names.iter().any(|n| n == "note");
        let has_edit_title = self.state_names.iter().any(|n| n == "edit_title");
        if has_editing && has_note_prop {
            script.push_str("onMounted(() => {\n");
            script.push_str("  if (!props.note?.title) {\n");
            if has_edit_title {
                script.push_str("    edit_title.value = ''\n");
                if self.state_names.iter().any(|n| n == "edit_body") {
                    script.push_str("    edit_body.value = ''\n");
                }
            }
            script.push_str("    editing.value = true\n");
            script.push_str("  }\n");
            script.push_str("})\n\n");
        }

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

        Ok(script)
    }

    /// Generate handler function body from LogicPayload
    fn generate_handler_body(&self, payload: &LogicPayload) -> GenResult<String> {
        match payload {
            LogicPayload::AstStmts(stmts) => {
                let mut ctx = crate::ui_gen::ts_adapter::AuraTsContext::new(self.state_names.iter().cloned().collect())
                    .with_props(self.prop_names.iter().cloned().collect());
                if !self.project_api_functions.is_empty() {
                    ctx = ctx.with_api_functions(self.project_api_functions.clone());
                }
                Ok(crate::ui_gen::ts_adapter::transpile_handler_body(stmts, &ctx))
            }
            LogicPayload::AstBlock(_) => {
                Err(GenError::UnsupportedStmt("AstBlock legacy path removed — use AstStmts".to_string()))
            }
            LogicPayload::Bytecode(_) => {
                Err(GenError::UnsupportedStmt("Bytecode not supported in Vue generator".to_string()))
            }
        }
    }

    /// Generate <template> content from view tree
    fn generate_template(&mut self, root: &AuraNode) -> GenResult<String> {
        let mut template = String::new();

        // Render the view root directly without an extra wrapper div.
        // The view's root element (col/row/etc.) already gets appropriate classes
        // from extract_classes(). Adding a hardcoded wrapper breaks apps that
        // need clean HTML (e.g., TodoMVC where body CSS controls layout).
        //
        // For dark mode, inject :class binding into the root element after generation.
        let root_html = self.node_to_html(root, 2)?;
        if self.has_dark_mode {
            // Add :class binding for isDark into the first opening tag
            let html = root_html.replacen("<div ", "<div :class=\"{ dark: isDark }\" ", 1);
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

                // Special handling for category-section element
                if tag == "category-section" || tag == "category_section" {
                    return self.generate_category_section_html(props, children, indent);
                }

                // Check if this is a known sub-widget (custom component, not shadcn)
                let is_known_sub_widget = self.known_sub_widgets.contains(tag);

                // Check if this is a shadcn-vue component
                // Note: We need to check both the original tag and lowercase version because registry uses lowercase keys
                let tag_lower = tag.to_lowercase();
                // If user provides a class prop on form elements, force native HTML
                // (e.g., TodoMVC needs <input type="checkbox" class="toggle"> not <Checkbox>)
                let has_user_class = props.contains_key("class") || props.contains_key("style");
                let force_native_elements = ["checkbox", "input", "button"];
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
                let is_shadcn_component = !is_known_sub_widget && !force_native && self.is_shadcn() &&
                    (self.widget_registry.is_backend_supported("vue", tag) ||
                     self.widget_registry.is_backend_supported("vue", &tag_lower));

                // For known sub-widgets, use component-style prop passing
                if is_known_sub_widget {
                    let mut attrs = Vec::new();
                    // Track first prop expression for :key binding
                    let mut first_prop_expr: Option<String> = None;
                    // Pass all props as v-bind (:prop="expr" needs a JS expression, not template text)
                    for (key, value) in props {
                        let value_str = match value {
                            AuraPropValue::Expr(expr) => self.expr_to_vue_bound_value(expr)?,
                            AuraPropValue::StyleBinding(_) => "\"\"".to_string(),
                        };
                        if first_prop_expr.is_none() {
                            first_prop_expr = Some(value_str.clone());
                        }
                        attrs.push(format!(":{}=\"{}\"", key, value_str));
                    }
                    // Add :key binding so sub-widget re-creates when data changes (e.g., switching notes)
                    if let Some(expr) = first_prop_expr {
                        attrs.push(format!(":key=\"{}?.id\"", expr));
                    }
                    // Event handlers
                    for (event, aura_event) in events {
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
                            html.push_str(&self.node_to_html(child, indent + 1)?);
                        }
                        html.push_str(&format!("{}</{}>\n", ind, html_tag));
                        return Ok(html);
                    }
                }

                // Build attributes
                let (attrs, text_content, generated_children) = if is_shadcn_component {
                    // Use shadcn-specific attribute generation (includes event handling)
                    let (shadcn_attrs, slot_content, slot_children) = self.generate_shadcn_attrs(tag, props, events);
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

                        // Handle two-way binding (bind:value -> v-model)
                        if key.starts_with("bind:") {
                            let bind_target = key.strip_prefix("bind:").unwrap();
                            let model_value = match value {
                                AuraPropValue::Expr(AuraExpr::StateRef(name)) => name.clone(),
                                _ => "value".to_string(),
                            };
                            if tag == "input" && bind_target == "checked" {
                                attrs.push(format!("v-model=\"{}\"", model_value));
                            } else if tag == "input" && bind_target == "value" {
                                attrs.push(format!("v-model=\"{}\"", model_value));
                            } else {
                                attrs.push(format!("v-model=\"{}\"", model_value));
                            }
                            continue;
                        }

                        // Use v-bind (:attr) for dynamic values, static quotes for literals
                        if let AuraPropValue::Expr(AuraExpr::StateRef(name)) = value {
                            // Track value state ref for v-model optimization on input elements
                            if key == "value" && (tag == "input" || tag == "textarea") {
                                value_state_ref = Some(name.clone());
                            }
                            attrs.push(format!(":{}=\"{}\"", key, name));
                        } else if let AuraPropValue::Expr(AuraExpr::FieldAccess { .. }) = value {
                            let value_str = self.prop_to_attr_value(value)?;
                            attrs.push(format!(":{}={}", key, value_str));
                        } else {
                            let value_str = self.prop_to_attr_value(value)?;
                            attrs.push(format!("{}={}", key, value_str));
                        }
                    }

                    // Event handlers
                    for (event, aura_event) in events {
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

                // Check if we have text content (render as inline content)
                if let Some(text) = &text_content {
                    if children.is_empty() {
                        // <button @click="handler">text</button>
                        Ok(format!("{}<{}{}>{}</{}>\n", ind, html_tag, attr_str, text, html_tag))
                    } else {
                        // Has both text and children - unusual but handle it
                        let mut html = format!("{}<{}{}>{}\n", ind, html_tag, attr_str, text);
                        for child in children {
                            html.push_str(&self.node_to_html(child, indent + 1)?);
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
                        html.push_str(&self.node_to_html(child, indent + 1)?);
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
                self.current_loop_var = Some(index.clone().unwrap_or_else(|| var.clone()));

                // If body has a single Element or Component, put v-for directly on it
                // to avoid <template> scoping issues with vue-tsc
                let result = if body.len() == 1 {
                    match &body[0] {
                        AuraNode::Element { .. } | AuraNode::Component { .. } => {
                            let child_html = self.node_to_html(&body[0], indent)?;
                            if let Some(gt_pos) = child_html.find('>') {
                                let mut result = child_html;
                                result.insert_str(gt_pos, &format!(" {}", v_for));
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
                Ok(format!("{}<div {}>\n{}{}</div>\n", ind, v_for, body_html, ind))
            }

            AuraNode::Conditional { condition, then_body, else_body, .. } => {
                // Convert condition to Vue expression
                let vue_condition = self.convert_condition(condition);

                let mut then_html = String::new();
                for child in then_body {
                    then_html.push_str(&self.node_to_html(child, indent + 1)?);
                }

                if let Some(else_nodes) = else_body {
                    let mut else_html = String::new();
                    for child in else_nodes {
                        else_html.push_str(&self.node_to_html(child, indent + 1)?);
                    }
                    Ok(format!(
                        "{}<template v-if=\"{}\">\n{}{}</template>\n{}<template v-else>\n{}{}</template>\n",
                        ind, vue_condition, then_html, ind, ind, else_html, ind
                    ))
                } else {
                    Ok(format!("{}<template v-if=\"{}\">\n{}{}</template>\n", ind, vue_condition, then_html, ind))
                }
            }

            AuraNode::Component { name, props, events, .. } => {
                // Build props as bindings
                let mut attrs = Vec::new();
                for (key, value) in props {
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
    fn expr_to_auto_string(&self, expr: &AuraExpr) -> String {
        match expr {
            AuraExpr::Int(n) => n.to_string(),
            AuraExpr::Float(n) => n.to_string(),
            AuraExpr::Bool(b) => b.to_string(),
            AuraExpr::Literal(s) => format!("\"{}\"", s),
            AuraExpr::StateRef(name) => format!(".{}", name),
            AuraExpr::MsgVariant { msg_type, variant } => {
                format!("{}::{}", msg_type, variant)
            }
            AuraExpr::Binary { left, op, right } => {
                let op_str = match op {
                    AuraBinOp::Add => "+",
                    AuraBinOp::Sub => "-",
                    AuraBinOp::Mul => "*",
                    AuraBinOp::Div => "/",
                    AuraBinOp::Mod => "%",
                    AuraBinOp::Eq => "==",
                    AuraBinOp::Ne => "!=",
                    AuraBinOp::Lt => "<",
                    AuraBinOp::Le => "<=",
                    AuraBinOp::Gt => ">",
                    AuraBinOp::Ge => ">=",
                    AuraBinOp::And => "&&",
                    AuraBinOp::Or => "||",
                };
                format!("{} {} {}", self.expr_to_auto_string(left), op_str, self.expr_to_auto_string(right))
            }
            AuraExpr::Unary { op, operand } => {
                let op_str = match op {
                    AuraUnaryOp::Neg => "-",
                    AuraUnaryOp::Not => "!",
                };
                format!("{}{}", op_str, self.expr_to_auto_string(operand))
            }
            AuraExpr::MethodCall { object, method, args } => {
                let args_str: Vec<String> = args.iter().map(|a| self.expr_to_auto_string(a)).collect();
                format!("{}.{}({})", self.expr_to_auto_string(object), method, args_str.join(", "))
            }
            AuraExpr::Array(elements) => {
                let elements_str: Vec<String> = elements.iter().map(|e| self.expr_to_auto_string(e)).collect();
                format!("[{}]", elements_str.join(", "))
            }
            AuraExpr::Object(fields) => {
                let pairs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.expr_to_auto_string(v)))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            AuraExpr::Lambda { params, body } => {
                let params_str = params.join(", ");
                format!("|{}| {}", params_str, self.expr_to_auto_string(body))
            }
            AuraExpr::FieldAccess { object, field } => {
                format!("{}.{}", self.expr_to_auto_string(object), field)
            }
            AuraExpr::NavCall { path, params } => {
                let params_str: Vec<String> = params.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.expr_to_auto_string(v)))
                    .collect();
                format!("Nav.to(\"{}\", {{ {} }})", path, params_str.join(", "))
            }
            AuraExpr::Constructor { type_name, args } => {
                let args_str: Vec<String> = args.iter().map(|a| self.expr_to_auto_string(a)).collect();
                format!("{}({})", type_name, args_str.join(", "))
            }
            AuraExpr::Index { target, index } => {
                format!("{}[{}]", self.expr_to_auto_string(target), self.expr_to_auto_string(index))
            }
            AuraExpr::If { cond, then_branch, else_branch } => {
                let else_str = else_branch.as_ref()
                    .map(|e| format!(" else {{ {} }}", self.expr_to_auto_string(e)))
                    .unwrap_or_default();
                format!("if {} {{ {} }}{}", self.expr_to_auto_string(cond), self.expr_to_auto_string(then_branch), else_str)
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
        // Priority: known sub-widgets > shadcn components > HTML fallback
        // If tag matches a known sub-widget, treat as custom component reference
        if self.known_sub_widgets.contains(tag) {
            if !self.component_refs.contains(&tag.to_string()) {
                self.component_refs.push(tag.to_string());
            }
            return tag.to_string();
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
                AuraPropValue::Expr(AuraExpr::Literal(s)) => format!("gap-{}", s),
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
                            format!("{}: {}", b.style_name, cond)
                        })
                        .collect();
                    dynamic_binding = Some(format!("{{ {} }}", binding_strs.join(", ")));
                }
                AuraPropValue::Expr(AuraExpr::Literal(s)) => {
                    // Dedup: for layout primitives, split user classes and skip any already present
                    if is_layout_primitive {
                        for c in s.split_whitespace() {
                            let existing: Vec<&str> = classes.iter().flat_map(|cl| cl.split_whitespace()).collect();
                            if !existing.contains(&c) {
                                classes.push(c.to_string());
                            }
                        }
                    } else {
                        classes.push(s.clone());
                    }
                }
                AuraPropValue::Expr(AuraExpr::If { cond, then_branch, else_branch }) => {
                    // Plan 346: conditional style → Vue :class ternary.
                    // style: if i == .active_index { "hl" } else { "normal" }
                    // → :class="i === active_index ? 'hl' : 'normal'"
                    let cond_str = self.expr_to_vue_bound_value(cond).unwrap_or_else(|_| "false".to_string());
                    let then_str = match then_branch.as_ref() {
                        AuraExpr::Literal(s) => s.clone(),
                        _ => String::new(),
                    };
                    let else_str = else_branch.as_ref()
                        .and_then(|e| match e.as_ref() {
                            AuraExpr::Literal(s) => Some(s.clone()),
                            _ => None,
                        })
                        .unwrap_or_default();
                    dynamic_binding = Some(format!("{} ? '{}' : '{}'", cond_str, then_str, else_str));
                }
                _ => {}
            }
        }
        // Process 'class' prop (static Tailwind classes)
        if let Some(value) = props.get("class") {
            match value {
                AuraPropValue::Expr(AuraExpr::Literal(s)) => {
                    if is_layout_primitive {
                        for c in s.split_whitespace() {
                            let existing: Vec<&str> = classes.iter().flat_map(|cl| cl.split_whitespace()).collect();
                            if !existing.contains(&c) {
                                classes.push(c.to_string());
                            }
                        }
                    } else {
                        classes.push(s.clone());
                    }
                }
                _ => {}
            }
        }

        (classes.join(" "), dynamic_binding)
    }

    /// Plan 100: Infer TypeScript type from AuraExpr
    fn expr_to_ts_type(&self, expr: &AuraExpr) -> String {
        match expr {
            AuraExpr::Int(_) => "number".to_string(),
            AuraExpr::Float(_) => "number".to_string(),
            AuraExpr::Bool(_) => "boolean".to_string(),
            AuraExpr::Literal(_) => "string".to_string(),
            AuraExpr::StateRef(name) => {
                // Try to infer type from state variable name
                // This is a simple heuristic - in a more complete implementation,
                // we'd look up the actual type from the state definition
                if name.starts_with("is_") || name.starts_with("has_") {
                    "boolean".to_string()
                } else {
                    "number".to_string()  // Default to number for state refs
                }
            }
            AuraExpr::Binary { .. } => {
                // Binary operations on numbers typically produce numbers
                // Comparison operations produce booleans
                "number".to_string()
            }
            AuraExpr::Unary { .. } => {
                "number".to_string()
            }
            AuraExpr::Array(_) => "any[]".to_string(),
            _ => "any".to_string(),  // Default fallback
        }
    }

    /// Convert Auto Type to TypeScript type string for defineProps
    fn type_to_ts_type(&self, ty: &crate::ast::Type) -> String {
        match ty {
            crate::ast::Type::Int | crate::ast::Type::Uint | crate::ast::Type::I64 | crate::ast::Type::U64 => "number".to_string(),
            crate::ast::Type::Float | crate::ast::Type::Double => "number".to_string(),
            crate::ast::Type::Bool => "boolean".to_string(),
            crate::ast::Type::StrSlice | crate::ast::Type::StrOwned | crate::ast::Type::CStrLit => "string".to_string(),
            crate::ast::Type::Array(_) | crate::ast::Type::RuntimeArray(_) | crate::ast::Type::List(_) => "any[]".to_string(),
            crate::ast::Type::User(decl) => decl.name.to_string(),
            crate::ast::Type::Enum(decl) => decl.borrow().name.to_string(),
            crate::ast::Type::Void => "void".to_string(),
            crate::ast::Type::Option(_) => "any".to_string(),
            crate::ast::Type::Map(_, _) => "Record<string, any>".to_string(),
            _ => "any".to_string(),
        }
    }

    /// Check if an expression contains NavCall (Plan 105)
    fn expr_has_nav_call(expr: &AuraExpr) -> bool {
        match expr {
            AuraExpr::NavCall { .. } => true,
            AuraExpr::Binary { left, right, .. } => {
                Self::expr_has_nav_call(left) || Self::expr_has_nav_call(right)
            }
            AuraExpr::Unary { operand, .. } => Self::expr_has_nav_call(operand),
            AuraExpr::MethodCall { object, args, .. } => {
                Self::expr_has_nav_call(object) || args.iter().any(Self::expr_has_nav_call)
            }
            AuraExpr::Array(elems) => elems.iter().any(Self::expr_has_nav_call),
            AuraExpr::Lambda { body, .. } => Self::expr_has_nav_call(body),
            AuraExpr::FieldAccess { object, .. } => Self::expr_has_nav_call(object),
            _ => false,
        }
    }

    /// Check if a statement contains NavCall (Plan 105)
    fn stmt_has_nav_call(stmt: &AuraStmt) -> bool {
        match stmt {
            AuraStmt::Assign { value, .. } => Self::expr_has_nav_call(value),
            AuraStmt::Update { value, .. } => Self::expr_has_nav_call(value),
            AuraStmt::MethodCall { args, .. } => args.iter().any(Self::expr_has_nav_call),
        }
    }

    /// Check if LogicPayload contains NavCall (Plan 105)
    fn payload_has_nav_call(payload: &LogicPayload) -> bool {
        match payload {
            LogicPayload::AstBlock(stmts) => stmts.iter().any(Self::stmt_has_nav_call),
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

    /// Check if an AURA expression accesses router (Plan 235)
    fn expr_has_route_access(expr: &AuraExpr) -> bool {
        match expr {
            AuraExpr::MethodCall { object, .. } => {
                if let AuraExpr::StateRef(name) = object.as_ref() {
                    if name == "router" {
                        return true;
                    }
                }
                Self::expr_has_route_access(object)
            }
            AuraExpr::FieldAccess { object, .. } => {
                if let AuraExpr::StateRef(name) = object.as_ref() {
                    if name == "router" {
                        return true;
                    }
                }
                Self::expr_has_route_access(object)
            }
            AuraExpr::Binary { left, right, .. } => {
                Self::expr_has_route_access(left) || Self::expr_has_route_access(right)
            }
            AuraExpr::Unary { operand, .. } => Self::expr_has_route_access(operand),
            AuraExpr::Array(elems) => elems.iter().any(Self::expr_has_route_access),
            AuraExpr::Object(fields) => fields.values().any(Self::expr_has_route_access),
            AuraExpr::Lambda { body, .. } => Self::expr_has_route_access(body),
            _ => false,
        }
    }

    /// Check if a statement contains router access (Plan 235)
    fn stmt_has_route_access(stmt: &AuraStmt) -> bool {
        match stmt {
            AuraStmt::Assign { value, .. } => Self::expr_has_route_access(value),
            AuraStmt::Update { value, .. } => Self::expr_has_route_access(value),
            AuraStmt::MethodCall { object, args, .. } => {
                if object == "router" {
                    return true;
                }
                args.iter().any(Self::expr_has_route_access)
            }
        }
    }

    /// Check if LogicPayload contains route access (Plan 235)
    fn payload_has_route_access(payload: &LogicPayload) -> bool {
        match payload {
            LogicPayload::AstBlock(stmts) => stmts.iter().any(Self::stmt_has_route_access),
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
    fn expr_to_js(&self, expr: &AuraExpr) -> GenResult<String> {
        match expr {
            AuraExpr::Literal(s) => Ok(format!("'{}'", Self::escape_js_string(s))),
            AuraExpr::Int(n) => Ok(n.to_string()),
            AuraExpr::Float(n) => Ok(n.to_string()),
            AuraExpr::Bool(b) => Ok(b.to_string()),
            AuraExpr::StateRef(name) => {
                if self.prop_names.contains(&name.to_string()) {
                    // Props: access via props.xxx (no .value, but need props. prefix in script)
                    Ok(format!("props.{}", name))
                } else if self.state_names.contains(&name.to_string()) {
                    Ok(format!("{}.value", name))
                } else {
                    Ok(name.clone())
                }
            }
            AuraExpr::Binary { left, op, right } => {
                let left_js = self.expr_to_js(left)?;
                let right_js = self.expr_to_js(right)?;
                let op_js = self.bin_op_to_js(op);
                Ok(format!("{} {} {}", left_js, op_js, right_js))
            }
            AuraExpr::Unary { op, operand } => {
                let operand_js = self.expr_to_js(operand)?;
                let op_js = match op {
                    crate::aura::AuraUnaryOp::Neg => "-",
                    crate::aura::AuraUnaryOp::Not => "!",
                };
                Ok(format!("{}{}", op_js, operand_js))
            }
            AuraExpr::MsgVariant { msg_type, variant } => {
                Ok(format!("{}.{}", msg_type, variant))
            }
            AuraExpr::MethodCall { object, method, args } => {
                // Plan 132: Check if this is an API function call
                // Case 1: Direct API call like listusers() - object is API name, method might be empty
                if let AuraExpr::StateRef(name) = object.as_ref() {
                    if self.is_api_function(&name) {
                        let args_js: Vec<String> = args.iter()
                            .map(|a| self.expr_to_js(a))
                            .collect::<Result<Vec<_>, _>>()?;
                        return Ok(format!("await {}({})", name, args_js.join(", ")));
                    }
                }
                // Case 2: self.<api_function>() - treat as direct API call
                // When AURA parser sees `listusers()` in handler, it converts to `self.listusers()`
                if let AuraExpr::StateRef(obj_name) = object.as_ref() {
                    if obj_name == "self" && self.is_api_function(&method) {
                        let args_js: Vec<String> = args.iter()
                            .map(|a| self.expr_to_js(a))
                            .collect::<Result<Vec<_>, _>>()?;
                        return Ok(format!("await {}({})", method, args_js.join(", ")));
                    }
                }
                // Case 3: Any method call where method name is an API function
                if self.is_api_function(&method) {
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
                    // for both strings and arrays (JS has no `.contains`).
                    "contains" => Ok(format!("{}.includes({})", object_js, args_js.join(", "))),
                    "to_string" => Ok(format!("{}.toString()", object_js)),
                    "to_int" => {
                        if args_js.is_empty() {
                            Ok(format!("parseInt({})", object_js))
                        } else {
                            Ok(format!("parseInt({}, {})", object_js, args_js.join(", ")))
                        }
                    }
                    _ => Ok(format!("{}.{}({})", object_js, method, args_js.join(", "))),
                }
            }
            AuraExpr::Array(elems) => {
                let elems_js: Vec<String> = elems.iter()
                    .map(|e| self.expr_to_js(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("[{}]", elems_js.join(", ")))
            }
            AuraExpr::Object(fields) => {
                let pairs_js: Vec<String> = fields.iter()
                    .map(|(k, v)| {
                        let v_js = self.expr_to_js(v)?;
                        Ok(format!("{}: {}", k, v_js))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{{{}}}", pairs_js.join(", ")))
            }
            AuraExpr::Lambda { params, body } => {
                let body_js = self.expr_to_js(body)?;
                Ok(format!("({}) => {}", params.join(", "), body_js))
            }
            AuraExpr::FieldAccess { object, field } => {
                let object_js = self.expr_to_js(object)?;
                Ok(format!("{}.{}", object_js, field))
            }
            AuraExpr::NavCall { path, params } => {
                if params.is_empty() {
                    // Simple path navigation: router.push('/path')
                    Ok(format!("router.push('{}')", path))
                } else {
                    // Navigation with query params: router.push({ path: '/path', query: { ... } })
                    let params_js: Vec<String> = params.iter()
                        .map(|(k, v)| {
                            self.expr_to_js(v).map(|v_js| format!("{}: {}", k, v_js))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(format!("router.push({{ path: '{}', query: {{ {} }} }})", path, params_js.join(", ")))
                }
            }
            AuraExpr::Constructor { type_name, args } => {
                let args_js: Vec<String> = args.iter()
                    .map(|a| self.expr_to_js(a))
                    .collect::<Result<Vec<_>, _>>()?;
                // In JavaScript, constructors use 'new' keyword
                Ok(format!("new {}({})", type_name, args_js.join(", ")))
            }
            AuraExpr::Index { target, index } => {
                let target_js = self.expr_to_js(target)?;
                let index_js = self.expr_to_js(index)?;
                Ok(format!("{}[{}]", target_js, index_js))
            }
            AuraExpr::If { cond, then_branch, else_branch } => {
                let cond_js = self.expr_to_js(cond)?;
                let then_js = self.expr_to_js(then_branch)?;
                let else_js = else_branch.as_ref()
                    .map(|e| self.expr_to_js(e))
                    .transpose()?
                    .unwrap_or_else(|| "undefined".to_string());
                Ok(format!("({} ? {} : {})", cond_js, then_js, else_js))
            }
            AuraExpr::If { .. } => Ok("undefined".to_string()),
            _ => Ok("undefined".to_string()),
        }
    }

    /// Convert AuraStmt to JS statement
    #[allow(dead_code)]
    fn stmt_to_js(&self, stmt: &AuraStmt) -> GenResult<String> {
        match stmt {
            AuraStmt::Assign { target, value } => {
                let value_js = self.expr_to_js(value)?;
                // Check if target is a ref
                if self.state_names.contains(target) {
                    Ok(format!("{}.value = {}", target, value_js))
                } else {
                    Ok(format!("{} = {}", target, value_js))
                }
            }
            AuraStmt::Update { target, op, value } => {
                let value_js = self.expr_to_js(value)?;
                let op_js = match op {
                    crate::aura::AuraUpdateOp::AddAssign => "+=",
                    crate::aura::AuraUpdateOp::SubAssign => "-=",
                    crate::aura::AuraUpdateOp::MulAssign => "*=",
                    crate::aura::AuraUpdateOp::DivAssign => "/=",
                };
                if self.state_names.contains(target) {
                    Ok(format!("{}.value {} {}", target, op_js, value_js))
                } else {
                    Ok(format!("{} {} {}", target, op_js, value_js))
                }
            }
            AuraStmt::MethodCall { object, method, args } => {
                let args_js: Vec<String> = args.iter()
                    .map(|a| self.expr_to_js(a))
                    .collect::<Result<Vec<_>, _>>()?;

                // print() → console.log() for browser
                if object.is_empty() && method == "print" {
                    return Ok(format!("console.log({})", args_js.join(", ")));
                }

                // Plan 132: Check if this is a standalone API function call
                // (object is API function name, method is likely empty or 'call')
                if self.is_api_function(&object) && method.is_empty() {
                    // Generate: await apiFunction(args)
                    return Ok(format!("await {}({})", object, args_js.join(", ")));
                }

                // Check if object is an API function being called as a method
                if self.is_api_function(&method) {
                    // This might be something like api.listusers() - should be await listusers()
                    return Ok(format!("await {}({})", method, args_js.join(", ")));
                }

                if self.state_names.contains(object) {
                    Ok(format!("{}.value.{}({})", object, method, args_js.join(", ")))
                } else {
                    Ok(format!("{}.{}({})", object, method, args_js.join(", ")))
                }
            }
        }
    }

    /// Convert binary operator to JS
    fn bin_op_to_js(&self, op: &crate::aura::AuraBinOp) -> &'static str {
        match op {
            crate::aura::AuraBinOp::Add => "+",
            crate::aura::AuraBinOp::Sub => "-",
            crate::aura::AuraBinOp::Mul => "*",
            crate::aura::AuraBinOp::Div => "/",
            crate::aura::AuraBinOp::Mod => "%",
            crate::aura::AuraBinOp::Eq => "===",
            crate::aura::AuraBinOp::Ne => "!==",
            crate::aura::AuraBinOp::Lt => "<",
            crate::aura::AuraBinOp::Le => "<=",
            crate::aura::AuraBinOp::Gt => ">",
            crate::aura::AuraBinOp::Ge => ">=",
            crate::aura::AuraBinOp::And => "&&",
            crate::aura::AuraBinOp::Or => "||",
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
            LogicPayload::AstBlock(stmts) => {
                for stmt in stmts {
                    self.extract_api_calls_from_stmt(stmt);
                }
            }
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

    /// Extract API function calls from a statement
    fn extract_api_calls_from_stmt(&mut self, stmt: &AuraStmt) {
        match stmt {
            AuraStmt::Assign { value, .. } => {
                self.extract_api_calls_from_expr(value);
            }
            AuraStmt::Update { value, .. } => {
                self.extract_api_calls_from_expr(value);
            }
            AuraStmt::MethodCall { args, .. } => {
                for arg in args {
                    self.extract_api_calls_from_expr(arg);
                }
            }
        }
    }

    /// Extract API function calls from an expression (recursive)
    fn extract_api_calls_from_expr(&mut self, expr: &AuraExpr) {
        match expr {
        AuraExpr::If { .. } => unreachable!(),
            AuraExpr::MethodCall { object, method, args } => {
                // Check if this is a direct API function call (e.g., listusers())
                if let AuraExpr::StateRef(name) = object.as_ref() {
                    if self.is_api_function(&name) {
                        self.api_functions_used.insert(name.clone());
                    }
                }
                // Also check if method name matches API function
                if self.is_api_function(&method) {
                    self.api_functions_used.insert(method.clone());
                }
                // Recurse into object and args
                self.extract_api_calls_from_expr(object);
                for arg in args {
                    self.extract_api_calls_from_expr(arg);
                }
            }
            AuraExpr::Binary { left, right, .. } => {
                self.extract_api_calls_from_expr(left);
                self.extract_api_calls_from_expr(right);
            }
            AuraExpr::Unary { operand, .. } => {
                self.extract_api_calls_from_expr(operand);
            }
            AuraExpr::FieldAccess { object, .. } => {
                self.extract_api_calls_from_expr(object);
            }
            AuraExpr::Array(elems) => {
                for elem in elems {
                    self.extract_api_calls_from_expr(elem);
                }
            }
            AuraExpr::Object(fields) => {
                for (_, v) in fields {
                    self.extract_api_calls_from_expr(v);
                }
            }
            AuraExpr::Lambda { body, .. } => {
                self.extract_api_calls_from_expr(body);
            }
            AuraExpr::NavCall { params, .. } => {
                for (_, v) in params {
                    self.extract_api_calls_from_expr(v);
                }
            }
            AuraExpr::Constructor { args, .. } => {
                for arg in args {
                    self.extract_api_calls_from_expr(arg);
                }
            }
            AuraExpr::Index { target, index } => {
                self.extract_api_calls_from_expr(target);
                self.extract_api_calls_from_expr(index);
            }
            // These don't contain nested expressions
            AuraExpr::Literal(_)
            | AuraExpr::Int(_)
            | AuraExpr::Float(_)
            | AuraExpr::Bool(_)
            | AuraExpr::StateRef(_)
            | AuraExpr::MsgVariant { .. } => {}
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
            LogicPayload::AstBlock(stmts) => {
                stmts.iter().any(|s| self.stmt_has_api_calls(s))
            }
            LogicPayload::Bytecode(_) => false,
        }
    }

    /// Check if a statement contains API calls
    fn stmt_has_api_calls(&self, stmt: &AuraStmt) -> bool {
        match stmt {
            AuraStmt::Assign { value, .. } => self.expr_has_api_calls(value),
            AuraStmt::Update { value, .. } => self.expr_has_api_calls(value),
            AuraStmt::MethodCall { object, method, args } => {
                // Check if method name is an API function
                if self.is_api_function(&method) {
                    return true;
                }
                // Check if object name is an API function (standalone call)
                if self.is_api_function(&object) {
                    return true;
                }
                // Check args
                args.iter().any(|a| self.expr_has_api_calls(a))
            }
        }
    }

    /// Check if an expression contains API calls (non-mutating version)
    fn expr_has_api_calls(&self, expr: &AuraExpr) -> bool {
        match expr {
        AuraExpr::If { .. } => unreachable!(),
            AuraExpr::MethodCall { object, method, args } => {
                // Check if method name is an API function
                if self.is_api_function(&method) {
                    return true;
                }
                // Check if object is an API function reference
                if let AuraExpr::StateRef(name) = object.as_ref() {
                    if self.is_api_function(&name) {
                        return true;
                    }
                }
                // Recurse
                self.expr_has_api_calls(object) || args.iter().any(|a| self.expr_has_api_calls(a))
            }
            AuraExpr::Binary { left, right, .. } => {
                self.expr_has_api_calls(left) || self.expr_has_api_calls(right)
            }
            AuraExpr::Unary { operand, .. } => self.expr_has_api_calls(operand),
            AuraExpr::FieldAccess { object, .. } => self.expr_has_api_calls(object),
            AuraExpr::Array(elems) => elems.iter().any(|e| self.expr_has_api_calls(e)),
            AuraExpr::Object(fields) => fields.values().any(|v| self.expr_has_api_calls(v)),
            AuraExpr::Lambda { body, .. } => self.expr_has_api_calls(body),
            AuraExpr::NavCall { params, .. } => {
                params.values().any(|v| self.expr_has_api_calls(v))
            }
            AuraExpr::Constructor { args, .. } => {
                args.iter().any(|a| self.expr_has_api_calls(a))
            }
            AuraExpr::Index { target, index } => {
                self.expr_has_api_calls(target) || self.expr_has_api_calls(index)
            }
            AuraExpr::Literal(_)
            | AuraExpr::Int(_)
            | AuraExpr::Float(_)
            | AuraExpr::Bool(_)
            | AuraExpr::StateRef(_)
            | AuraExpr::MsgVariant { .. } => false,
        }
    }

    /// Convert prop value to HTML attribute value
    /// For static values: produces `"value"`
    /// For dynamic values (StateRef, FieldAccess): produces `"name"` (caller must prefix with `:`)
    fn prop_to_attr_value(&self, value: &AuraPropValue) -> GenResult<String> {
        match value {
            AuraPropValue::Expr(expr) => {
                match expr {
                    AuraExpr::StateRef(name) => Ok(format!("\"{}\"", name)),
                    AuraExpr::FieldAccess { object, field } => {
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
    fn expr_to_vue_text_raw(&self, expr: &AuraExpr) -> GenResult<String> {
        match expr {
            AuraExpr::Literal(s) => {
                // Check if this is a template string with ${...} placeholders
                // Strip outer {{ }} if present, since caller will wrap
                let vue = self.convert_template_to_vue(s);
                // Only strip {{ }} if the ENTIRE string is wrapped in them.
                // Using trim_end_matches('}') is too aggressive — it would strip
                // the closing }} from embedded Vue interpolations like "Counter: {{ count }}"
                let vue = vue.strip_prefix("{{ ")
                    .and_then(|v| v.strip_suffix(" }}"))
                    .map(|v| v.to_string())
                    .unwrap_or(vue);
                Ok(vue)
            }
            AuraExpr::Int(n) => Ok(n.to_string()),
            AuraExpr::Float(f) => Ok(f.to_string()),
            AuraExpr::Bool(b) => Ok(b.to_string()),
            AuraExpr::StateRef(name) => Ok(name.clone()),
            AuraExpr::FieldAccess { object, field } => {
                // Handle user.name -> user.name
                let object_str = self.expr_to_vue_text_raw(object)?;
                Ok(format!("{}.{}", object_str, field))
            }
            AuraExpr::Index { target, index } => {
                let target_str = self.expr_to_vue_text_raw(target)?;
                let index_str = self.expr_to_vue_text_raw(index)?;
                Ok(format!("{}[{}]", target_str, index_str))
            }
            AuraExpr::Binary { left, op: _, right } => {
                let left_str = self.expr_to_vue_text_raw(left)?;
                let right_str = self.expr_to_vue_text_raw(right)?;
                Ok(format!("{}{}", left_str, right_str))
            }
            AuraExpr::MethodCall { object, method, args } => {
                let obj_str = self.expr_to_vue_text_raw(object)?;
                let is_self = obj_str == "self";
                match method.as_str() {
                    "to_string" => Ok(obj_str.clone()),
                    "len" => Ok(format!("{}.length", obj_str)),
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
            }
            _ => Ok("value".to_string()),
        }
    }

    /// Convert AuraExpr to Vue template text with {{ }} wrapping for display.
    /// Uses expr_to_vue_text_raw internally and wraps the final result.
    fn expr_to_vue_text(&self, expr: &AuraExpr) -> GenResult<String> {
        // For compound expressions that produce their own {{ }},
        // use the raw version and wrap at the end.
        let raw = self.expr_to_vue_text_raw(expr)?;
        // If the raw result already contains {{ (e.g., from convert_template_to_vue),
        // or is a plain literal string, return as-is.
        // Otherwise wrap in {{ }}.
        if raw.starts_with("{{") || matches!(expr, AuraExpr::Literal(_)) {
            Ok(raw)
        } else {
            Ok(format!("{{{{ {} }}}}", raw))
        }
    }

    /// Convert AuraExpr to Vue bound attribute value (for :prop="..." bindings).
    /// Used for chart props and other complex bindings where we need JavaScript
    /// expressions in Vue templates (state refs are kept bare, no .value).
    fn expr_to_vue_bound_value(&self, expr: &AuraExpr) -> GenResult<String> {
        match expr {
            AuraExpr::Literal(s) => Ok(format!("'{}'", Self::escape_js_string(s))),
            AuraExpr::Int(n) => Ok(n.to_string()),
            AuraExpr::Float(n) => Ok(n.to_string()),
            AuraExpr::Bool(b) => Ok(b.to_string()),
            AuraExpr::StateRef(name) => Ok(name.clone()),
            AuraExpr::FieldAccess { object, field } => {
                let obj_str = self.expr_to_vue_bound_value(object)?;
                Ok(format!("{}.{}", obj_str, field))
            }
            AuraExpr::Index { target, index } => {
                let target_str = self.expr_to_vue_bound_value(target)?;
                let index_str = self.expr_to_vue_bound_value(index)?;
                Ok(format!("{}[{}]", target_str, index_str))
            }
            AuraExpr::Array(elems) => {
                let elems_vue: Vec<String> = elems.iter()
                    .map(|e| self.expr_to_vue_bound_value(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("[{}]", elems_vue.join(", ")))
            }
            AuraExpr::Object(fields) => {
                let pairs_vue: Vec<String> = fields.iter()
                    .map(|(k, v)| {
                        let v_vue = self.expr_to_vue_bound_value(v)?;
                        Ok(format!("{}: {}", k, v_vue))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{{{}}}", pairs_vue.join(", ")))
            }
            AuraExpr::Binary { left, op, right } => {
                let left_str = self.expr_to_vue_bound_value(left)?;
                let right_str = self.expr_to_vue_bound_value(right)?;
                let op_str = match op {
                    AuraBinOp::Eq => "==",
                    AuraBinOp::Ne => "!=",
                    AuraBinOp::Lt => "<",
                    AuraBinOp::Gt => ">",
                    AuraBinOp::Le => "<=",
                    AuraBinOp::Ge => ">=",
                    AuraBinOp::And => "&&",
                    AuraBinOp::Or => "||",
                    AuraBinOp::Add => "+",
                    AuraBinOp::Sub => "-",
                    AuraBinOp::Mul => "*",
                    AuraBinOp::Div => "/",
                    AuraBinOp::Mod => "%",
                };
                Ok(format!("{} {} {}", left_str, op_str, right_str))
            }
            AuraExpr::Unary { op, operand } => {
                let expr_str = self.expr_to_vue_bound_value(operand)?;
                match op {
                    AuraUnaryOp::Not => Ok(format!("!{}", expr_str)),
                    AuraUnaryOp::Neg => Ok(format!("-{}", expr_str)),
                }
            }
            _ => Ok("null".to_string()),
        }
    }

    /// Emit a chart prop attribute.
    /// Literal strings become static attributes; everything else becomes a bound attribute.
    fn emit_chart_prop(&mut self, attrs: &mut Vec<String>, props: &HashMap<String, AuraPropValue>, key: &str, vue_attr: &str) {
        if let Some(value) = props.get(key) {
            match value {
                AuraPropValue::Expr(AuraExpr::Literal(s)) => {
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
            } else if let AuraPropValue::Expr(AuraExpr::StateRef(name)) = value {
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

            // === Input ===
            "input" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
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
                        attrs.push(format!("v-model=\"{}\"", model));
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
            "modal" => {
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
                            AuraPropValue::Expr(AuraExpr::StateRef(name)) => {
                                attrs.push(format!(":{}=\"{}\"", key, name));
                            }
                            AuraPropValue::Expr(AuraExpr::FieldAccess { .. }) => {
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
                    if let AuraPropValue::Expr(AuraExpr::StateRef(name)) = value {
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
                    if let AuraPropValue::Expr(AuraExpr::StateRef(name)) = value {
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
                    if let AuraPropValue::Expr(AuraExpr::StateRef(name)) = value {
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
                    if let AuraPropValue::Expr(AuraExpr::StateRef(name)) = value {
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
        match event {
            "onclick" | "onClick" | "on_click" => "@click".to_string(),
            "oninput" | "onInput" => "@update:modelValue".to_string(),
            "onchange" | "onChange" => "@update:modelValue".to_string(),
            "onenter" | "onEnter" => "@keyup.enter".to_string(),
            _ => format!("@{}", event.trim_start_matches("on")),
        }
    }

    /// Extract string value from AuraPropValue
    fn extract_string_value<'a>(&self, value: &'a AuraPropValue) -> Option<&'a str> {
        match value {
            AuraPropValue::Expr(AuraExpr::Literal(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Extract boolean value from AuraPropValue
    fn extract_bool_value(&self, value: &AuraPropValue) -> bool {
        match value {
            AuraPropValue::Expr(AuraExpr::Bool(b)) => *b,
            AuraPropValue::Expr(AuraExpr::Literal(s)) => s == "true",
            _ => false,
        }
    }

    /// Extract integer value from AuraPropValue
    fn extract_int_value(&self, value: &AuraPropValue) -> Option<i64> {
        match value {
            AuraPropValue::Expr(AuraExpr::Int(n)) => Some(*n),
            AuraPropValue::Expr(AuraExpr::Literal(s)) => s.parse().ok(),
            _ => None,
        }
    }

    /// Extract float value from AuraPropValue
    fn extract_float_value(&self, value: &AuraPropValue) -> Option<f64> {
        match value {
            AuraPropValue::Expr(AuraExpr::Float(n)) => Some(*n),
            AuraPropValue::Expr(AuraExpr::Int(n)) => Some(*n as f64),
            AuraPropValue::Expr(AuraExpr::Literal(s)) => s.parse().ok(),
            _ => None,
        }
    }

    /// Extract state reference from AuraPropValue
    fn extract_state_ref(&self, value: &AuraPropValue) -> Option<String> {
        match value {
            AuraPropValue::Expr(AuraExpr::StateRef(name)) => Some(name.clone()),
            _ => None,
        }
    }

    /// Get style or class prop value (style takes priority over class)
    /// This supports the transition from 'class' to 'style' prop naming
    fn get_style_class<'a>(&self, props: &'a HashMap<String, AuraPropValue>) -> Option<&'a AuraPropValue> {
        props.get("style").or_else(|| props.get("class"))
    }

    /// Convert AutoUI event name to Vue event
    fn auto_event_to_vue(&self, event: &str) -> String {
        match event {
            "onclick" | "onClick" | "on_click" => "@click".to_string(),
            "oninput" | "onInput" => "@input".to_string(),
            "onchange" | "onChange" => "@change".to_string(),
            "onenter" | "onEnter" => "@keyup.enter".to_string(),
            "onblur" | "onBlur" => "@blur".to_string(),
            "ondblclick" | "onDblClick" | "on_double_click" => "@dblclick".to_string(),
            "onsubmit" | "onSubmit" => "@submit.prevent".to_string(),
            _ => format!("@{}", event.trim_start_matches("on")),
        }
    }

    /// Convert handler pattern to function name
    fn pattern_to_handler_name(&self, pattern: &str) -> String {
        // Check for dot prefix first (e.g., ".Inc")
        if pattern.starts_with('.') {
            // Dot-prefixed handlers map directly to function name (Vue convention)
            pattern[1..].to_string()
        } else if let Some(variant) = pattern.split("::").last() {
            // Pattern like "Msg::Inc" -> "onInc"
            format!("on{}", variant)
        } else {
            format!("on{}", pattern)
        }
    }

    /// Convert handler reference to function call
    fn handler_to_function_call(&self, handler: &str) -> String {
        // Check for dot prefix first
        if handler.starts_with('.') {
            // Dot-prefixed handlers map directly to function name (Vue convention)
            handler[1..].to_string()
        } else if let Some(variant) = handler.split("::").last() {
            // Handler like "Msg::Inc" -> "onInc"
            format!("on{}", variant)
        } else {
            format!("on{}", handler)
        }
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
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
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
  }

  .dark {
    --background: 222.2 84% 4.9%;
    --foreground: 210 40% 98%;
    --card: 222.2 84% 4.9%;
    --card-foreground: 210 40% 98%;
    --popover: 222.2 84% 4.9%;
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
  }
}

@layer base {
  * {
    @apply border-border;
  }
  body {
    @apply bg-background text-foreground;
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
        code.push_str("import { ref } from 'vue'\n\n");

        // Module-level ref declarations (singleton state).
        for sv in &store.state_vars {
            let init = Self::store_init_to_js(&sv.initial);
            code.push_str(&format!("const {} = ref({})\n", sv.name, init));
        }
        code.push('\n');

        // Build ctx for handler transpilation (state_names → .value emission).
        let state_names: std::collections::HashSet<String> =
            store.state_vars.iter().map(|s| s.name.clone()).collect();
        let ctx = AuraTsContext::new(state_names)
            .with_props(std::collections::HashSet::new());

        // Export function.
        let fn_name = format!("use{}Store", store.name);
        code.push_str(&format!("export function {}() {{\n", fn_name));
        code.push_str("    return {\n");

        // Expose state refs by name.
        for sv in &store.state_vars {
            code.push_str(&format!("        {},\n", sv.name));
        }

        // Expose handlers as action functions.
        for (pattern, payload) in &store.handlers {
            let action_name = pattern.trim_start_matches('.');
            let body = match payload {
                crate::aura::LogicPayload::AstStmts(stmts) => transpile_handler_body(stmts, &ctx),
                _ => String::new(),
            };
            code.push_str(&format!(
                "        {}: () => {{ {} }},\n",
                action_name, body
            ));
        }

        code.push_str("    }\n");
        code.push_str("}\n");
        code
    }

    /// Convert an initial-value AuraExpr to a JS literal (v1: simple cases).
    fn store_init_to_js(expr: &crate::aura::AuraExpr) -> String {
        use crate::aura::AuraExpr;
        match expr {
            AuraExpr::Int(n) => n.to_string(),
            AuraExpr::Literal(s) => format!("'{}'", s.replace('\'', "\\'")),
            AuraExpr::Bool(b) => b.to_string(),
            AuraExpr::Array(_) => "[]".to_string(),
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
    "badge",
    "avatar",
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
                initial: AuraExpr::Int(0),
                decorators: vec![],
            }],
            messages: vec![AuraMessage {
                name: "Msg".to_string(),
                variants: vec![
                    AuraMsgVariant { name: "Inc".to_string(), payload: None },
                    AuraMsgVariant { name: "Dec".to_string(), payload: None },
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

    #[test]
    fn test_expr_to_js() {
        let gen = VueGenerator::new();

        assert_eq!(gen.expr_to_js(&AuraExpr::Int(42)).unwrap(), "42");
        assert_eq!(gen.expr_to_js(&AuraExpr::Bool(true)).unwrap(), "true");
        assert_eq!(gen.expr_to_js(&AuraExpr::Literal("hello".to_string())).unwrap(), "'hello'");
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

        props.insert("data".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("monthlyRevenue".to_string())));
        props.insert("categories".to_string(), AuraPropValue::Expr(AuraExpr::Array(vec![
            AuraExpr::Literal("desktop".to_string()),
            AuraExpr::Literal("mobile".to_string()),
        ])));
        props.insert("index".to_string(), AuraPropValue::Expr(AuraExpr::Literal("month".to_string())));
        props.insert("show-x-axis".to_string(), AuraPropValue::Expr(AuraExpr::Bool(false)));
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

        props.insert("data".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("quarterlySales".to_string())));
        props.insert("type".to_string(), AuraPropValue::Expr(AuraExpr::Literal("stacked".to_string())));
        props.insert("rounded-corners".to_string(), AuraPropValue::Expr(AuraExpr::Bool(true)));
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

        props.insert("curve-type".to_string(), AuraPropValue::Expr(AuraExpr::Literal("monotone".to_string())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("line-chart", &props, &events);

        assert!(attrs.iter().any(|a| a.contains(":curve-type=\"CurveType.MonotoneX\"")));
        assert!(gen.use_curve_type);
    }

    #[test]
    fn test_generate_shadcn_attrs_donut_chart() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        props.insert("category".to_string(), AuraPropValue::Expr(AuraExpr::Literal("source".to_string())));
        props.insert("value-formatter".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("formatValue".to_string())));
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
        props.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Click me".to_string())));
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
        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("name".to_string())));
        props.insert("placeholder".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Enter name".to_string())));
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
        props.insert("checked".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("done".to_string())));
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
        props.insert("orientation".to_string(), AuraPropValue::Expr(AuraExpr::Literal("vertical".to_string())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("scroll", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("orientation=\"vertical\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_tabs() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tabs with default value
        props.insert("default".to_string(), AuraPropValue::Expr(AuraExpr::Literal("tab1".to_string())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("tabs", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("default-value=\"tab1\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_tabs_with_model() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tabs with v-model
        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("activeTab".to_string())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("tabs", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("v-model=\"activeTab\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_tab() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tab trigger with value and text
        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::Literal("tab1".to_string())));
        props.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("First Tab".to_string())));
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
        props.insert("variant".to_string(), AuraPropValue::Expr(AuraExpr::Literal("outline".to_string())));
        props.insert("title".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Card Title".to_string())));
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
        props.insert("orientation".to_string(), AuraPropValue::Expr(AuraExpr::Literal("vertical".to_string())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("divider", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("orientation=\"vertical\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_divider_decorative() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test decorative separator
        props.insert("decorative".to_string(), AuraPropValue::Expr(AuraExpr::Bool(true)));
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
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test modal with v-model:open
        props.insert("open".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("showDialog".to_string())));
        props.insert("title".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Confirm Delete".to_string())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("modal", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("v-model:open=\"showDialog\"")));
        assert!(attrs.iter().any(|a| a.contains("data-title=\"Confirm Delete\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_tooltip() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tooltip with content and side
        props.insert("content".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Help text".to_string())));
        props.insert("side".to_string(), AuraPropValue::Expr(AuraExpr::Literal("right".to_string())));
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
        props.insert("class".to_string(), AuraPropValue::Expr(AuraExpr::Literal("w-10 h-10".to_string())));
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
        props.insert("class".to_string(), AuraPropValue::Expr(AuraExpr::Literal("w-full".to_string())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("table", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("class=\"w-full\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_table_cells() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test th with colspan
        props.insert("colspan".to_string(), AuraPropValue::Expr(AuraExpr::Int(2)));
        let (attrs, _, _) = gen.generate_shadcn_attrs("th", &props, &events);

        assert!(attrs.iter().any(|a| a.contains(":colspan=\"2\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_tree() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tree
        props.insert("class".to_string(), AuraPropValue::Expr(AuraExpr::Literal("pl-4".to_string())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("tree", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("class=\"pl-4\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_tree_item() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tree_item with text
        props.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Node 1".to_string())));
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
        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("selectedOption".to_string())));
        props.insert("name".to_string(), AuraPropValue::Expr(AuraExpr::Literal("options".to_string())));
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
        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::Literal("option1".to_string())));
        props.insert("label".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Option 1".to_string())));
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
        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::Literal("option2".to_string())));
        props.insert("disabled".to_string(), AuraPropValue::Expr(AuraExpr::Bool(true)));
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
        use crate::aura::AuraExpr;
        use std::collections::HashMap;

        // Create a simple Button element node
        let button_node = AuraNode::Element {
            tag: "Button".to_string(),
            props: {
                let mut map = HashMap::new();
                map.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Click Me".to_string())));
                map.insert("style".to_string(), AuraPropValue::Expr(AuraExpr::Literal("px-4 py-2 bg-blue-500".to_string())));
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
        };

        let mut gen = VueGenerator::new_shadcn();
        let vue_code = gen.generate(&widget).unwrap();

        // Check that the button is NOT self-closing and has text content
        assert!(vue_code.contains("<Button") && vue_code.contains("Click Me"));
        // Should NOT be self-closing (should have >Click Me< pattern)
        assert!(vue_code.contains("Click Me") && vue_code.contains("</Button>"));
    }
}
