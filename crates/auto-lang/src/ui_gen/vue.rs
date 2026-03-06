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
//! <style scoped>
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

use super::{BackendGenerator, GenError, GenResult};
use crate::aura::{AuraBinOp, AuraEvent, AuraExpr, AuraNode, AuraPropValue, AuraStateDef, AuraStmt, AuraTextContent, AuraUnaryOp, AuraWidget, LogicPayload};
use std::collections::{HashMap, HashSet};

// ============================================================================
// shadcn-vue Component Registry
// ============================================================================

/// Maps AURA element tags to shadcn-vue component imports
pub struct ShadcnRegistry {
    /// Component imports needed: tag -> (module_path, component_names)
    components: HashMap<&'static str, (&'static str, Vec<&'static str>)>,
}

impl ShadcnRegistry {
    /// Create registry with all shadcn-vue component mappings
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
}

/// Vue3 SFC generator
pub struct VueGenerator {
    /// Current widget name
    current_widget: Option<String>,

    /// Collected imports
    imports: Vec<String>,

    /// State variable names (for ref() detection)
    state_names: Vec<String>,

    /// Event handler definitions
    handlers: Vec<(String, String)>,

    /// Event names for emit
    emit_events: Vec<String>,

    /// Whether emit is needed
    has_emit: bool,

    /// Component references (other widgets)
    component_refs: Vec<String>,

    /// Tailwind classes for wrapper
    wrapper_classes: String,

    /// Generation mode (Plain or Shadcn)
    mode: VueMode,

    /// shadcn-vue component registry
    shadcn_registry: ShadcnRegistry,

    /// Track which shadcn-vue components are used
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
            handlers: Vec::new(),
            emit_events: Vec::new(),
            has_emit: false,
            component_refs: Vec::new(),
            wrapper_classes: String::new(),
            mode: VueMode::Plain,
            shadcn_registry: ShadcnRegistry::new(),
            shadcn_components_used: HashSet::new(),
            use_typescript: true,  // Plan 100: TypeScript by default
            previewcard_counter: 0,
            previewcard_data: Vec::new(),
            needs_copy_code: false,
            codeblock_counter: 0,
            codeblock_data: Vec::new(),
            needs_router: false,
        }
    }

    /// Create a new Vue generator in shadcn-vue mode
    pub fn new_shadcn() -> Self {
        Self {
            mode: VueMode::Shadcn,
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

    /// Check if outputting TypeScript (Plan 100)
    pub fn is_typescript(&self) -> bool {
        self.use_typescript
    }

    /// Reset state for new widget
    fn reset(&mut self) {
        self.imports.clear();
        self.state_names.clear();
        self.handlers.clear();
        self.emit_events.clear();
        self.has_emit = false;
        self.component_refs.clear();
        self.wrapper_classes.clear();
        self.shadcn_components_used.clear();
        self.previewcard_counter = 0;
        self.previewcard_data.clear();
        self.needs_copy_code = false;
        self.codeblock_counter = 0;
        self.codeblock_data.clear();
        self.needs_router = false;
    }

    /// Generate complete Vue3 SFC
    pub fn generate_sfc(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.current_widget = Some(widget.name.clone());
        self.reset();

        // Generate template first to collect shadcn components used and detect Outlet/Link
        let template = self.generate_template(&widget.view_tree)?;

        // Plan 105: Check handlers for NavCall
        if self.widget_needs_router(widget) {
            self.needs_router = true;
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

<style scoped>
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
        // Plan 106: Add watch and nextTick for Prism.js re-highlighting
        if !self.previewcard_data.is_empty() {
            imports.push("watch");
            imports.push("nextTick");
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

        // Generate shadcn-vue imports (if any components were used in template)
        let shadcn_imports = self.generate_shadcn_imports();
        if !shadcn_imports.is_empty() {
            script.push_str(&shadcn_imports);
            script.push('\n');
        }
        script.push('\n');

        // Generate state variables as ref()
        for state in &widget.state_vars {
            self.state_names.push(state.name.clone());
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

        // Generate emit if needed
        if self.has_emit {
            script.push_str("const emit = defineEmits<{\n");
            for event in &self.emit_events {
                script.push_str(&format!("  {}: []\n", event));
            }
            script.push_str("}>()\n\n");
        }

        // Generate event handlers
        for (pattern, payload) in &widget.handlers {
            let handler_name = self.pattern_to_handler_name(pattern);
            let body = self.generate_handler_body(payload)?;
            self.handlers.push((handler_name.clone(), body));
        }

        // Output handler functions
        // Plan 100: Add return type annotation for TypeScript
        let return_type = if self.use_typescript { ": void" } else { "" };
        for (handler_name, handler_body) in &self.handlers {
            if handler_body.is_empty() {
                script.push_str(&format!("function {}(){} {{\n  // TODO\n}}\n\n", handler_name, return_type));
            } else {
                script.push_str(&format!("function {}(){} {{\n  {}\n}}\n\n", handler_name, return_type, handler_body));
            }
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
            LogicPayload::AstBlock(stmts) => {
                let body: Vec<String> = stmts.iter()
                    .map(|s| self.stmt_to_js(s))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(body.join("\n  "))
            }
            LogicPayload::Bytecode(_) => {
                Err(GenError::UnsupportedStmt("Bytecode not supported in Vue generator".to_string()))
            }
        }
    }

    /// Generate <template> content from view tree
    fn generate_template(&mut self, root: &AuraNode) -> GenResult<String> {
        let mut template = String::new();

        // Wrapper div with Tailwind classes
        let classes = if self.wrapper_classes.is_empty() {
            "flex flex-col".to_string()
        } else {
            format!("flex flex-col {}", self.wrapper_classes)
        };

        template.push_str(&format!("  <div class=\"{}\">\n", classes));
        template.push_str(&self.node_to_html(root, 2)?);
        template.push_str("  </div>\n");

        Ok(template)
    }

    /// Generate <style scoped> content
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
            AuraNode::Element { tag, props, events, children } => {
                // Special handling for previewcard element (supports both previewcard and preview-card)
                if tag == "previewcard" || tag == "preview-card" {
                    return self.generate_previewcard_html(props, events, children, indent);
                }

                // Special handling for codeblock element (with copy button)
                if tag == "codeblock" || tag == "code-block" {
                    return self.generate_codeblock_html(props, events, children, indent);
                }

                let html_tag = self.map_tag(tag, children.is_empty());

                // Check if this is a shadcn-vue component
                let is_shadcn_component = self.is_shadcn() && self.shadcn_registry.has_component(tag);

                // Build attributes
                let (attrs, text_content, generated_children) = if is_shadcn_component {
                    // Use shadcn-specific attribute generation
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

                    // Props as attributes
                    for (key, value) in props {
                        if key == "class" {
                            continue; // Already handled
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

                        let value_str = self.prop_to_attr_value(value)?;
                        attrs.push(format!("{}={}", key, value_str));
                    }

                    // Event handlers
                    for (event, aura_event) in events {
                        let vue_event = self.auto_event_to_vue(event);
                        let handler_fn = self.handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
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

            AuraNode::ForLoop { var, index, iterable, body } => {
                // Generate v-for directive
                // Auto syntax: for idx, item in list (index first, value second)
                // Vue syntax: v-for="(item, index) in list" (value first, index second)
                // So we need to swap the order for Vue
                let v_for = if let Some(idx) = index {
                    format!("v-for=\"({}, {}) in {}\"", var, idx, iterable.trim_start_matches('.'))
                } else {
                    format!("v-for=\"{} in {}\"", var, iterable.trim_start_matches('.'))
                };

                // Wrap body in a container with v-for
                let mut body_html = String::new();
                for child in body {
                    body_html.push_str(&self.node_to_html(child, indent + 1)?);
                }

                // Use template tag for the loop wrapper
                Ok(format!("{}<template {}>\n{}{}</template>\n", ind, v_for, body_html, ind))
            }

            AuraNode::Conditional { condition, then_body, else_body } => {
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

            AuraNode::Component { name, props, events } => {
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

            AuraNode::Link { to, text, href, children } => {
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
                    Ok(format!("{}<router-link to=\"{}\">\n{}{}</router-link>\n", ind, to, children_html, ind))
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
{ind}      <pre class="overflow-x-auto p-4 text-sm bg-zinc-950 text-zinc-50"><code :class="'block font-mono !p-0 language-' + active{id_cap}Tab.toLowerCase()">{{{{ active{id_cap}Tab === 'auto' ? {id_lower}AutoCode : {id_lower}VueCode }}}}</code></pre>
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
            AuraNode::Element { tag, props, events, children } => {
                let mut result = String::new();

                // Build props string
                let mut props_parts = Vec::new();
                for (key, value) in props {
                    let value_str = match value {
                        AuraPropValue::Expr(expr) => self.expr_to_auto_string(expr),
                        AuraPropValue::ClassBinding(bindings) => {
                            let binding_strs: Vec<String> = bindings.iter()
                                .map(|b| format!("{}: {}", b.class_name, self.expr_to_auto_string(&b.condition)))
                                .collect();
                            format!("{{{}}}", binding_strs.join(", "))
                        }
                    };
                    props_parts.push(format!("{}: {}", key, value_str));
                }

                // Build events string
                for (event_name, event) in events {
                    let params_str = if event.params.is_empty() {
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
                    AuraTextContent::Interpolated { template, bindings } => {
                        // Show the template with bindings
                        format!("{}\"{}\"\n", ind, template)
                    }
                }
            }

            AuraNode::Conditional { condition, then_body, else_body } => {
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

            AuraNode::ForLoop { var, index, iterable, body } => {
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

            AuraNode::Component { name, props, events } => {
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

            AuraNode::Link { to, text, href, children } => {
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
        }
    }

    /// Convert AURA condition to Vue expression
    fn convert_condition(&mut self, condition: &str) -> String {
        // Convert .var to var, .len to .length, etc.
        let mut result = condition.trim().to_string();

        // Replace .len with .length
        result = result.replace(".len", ".length");

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

        final_result
    }

    /// Map AutoUI tag to HTML tag or shadcn-vue component
    fn map_tag(&mut self, tag: &str, self_closing: bool) -> String {
        // If in shadcn mode and tag has a shadcn component, use it
        if self.is_shadcn() {
            if let Some(component_name) = self.shadcn_component_name(tag) {
                self.register_shadcn_component(tag);
                return component_name.to_string();
            }
        }

        // Fallback to plain HTML tags
        match tag {
            // Layout (no shadcn components, use Tailwind)
            "col" | "column" => "div".to_string(),
            "row" => "div".to_string(),
            "grid" => "div".to_string(),
            "scroll" => "div".to_string(),
            "container" => "div".to_string(),
            "center" => "div".to_string(),

            // HTML5 semantic elements
            "header" => "header".to_string(),
            "nav" => "nav".to_string(),
            "main" => "main".to_string(),
            "section" => "section".to_string(),
            "aside" => "aside".to_string(),
            "footer" => "footer".to_string(),
            "article" => "article".to_string(),

            // Content
            "button" => "button".to_string(),
            "input" => "input".to_string(),
            "textarea" => "textarea".to_string(),
            "checkbox" => "input".to_string(),
            "toggle" => "button".to_string(),
            "select" => "select".to_string(),
            "option" => "option".to_string(),
            "link" => "a".to_string(),
            "codeblock" | "code-block" => "pre".to_string(),
            "codepane" | "code-pane" => "div".to_string(),
            "previewcard" | "preview-card" => "div".to_string(),

            // Typography (no shadcn components)
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => tag.to_string(),
            "text" | "label" | "span" => "span".to_string(),
            "p" => "p".to_string(),

            // Data
            "table" => "table".to_string(),
            "thead" => "thead".to_string(),
            "tbody" => "tbody".to_string(),
            "tr" => "tr".to_string(),
            "th" => "th".to_string(),
            "td" => "td".to_string(),
            "tree" => "ul".to_string(),
            "tree_item" | "tree-item" => "li".to_string(),

            // Navigation
            "tabs" => "div".to_string(),
            "tab" => "button".to_string(),

            // Overlay
            "modal" => "div".to_string(),
            "tooltip" => "span".to_string(),

            // Form
            "slider" => "input".to_string(),
            "radio" => "input".to_string(),
            "radiogroup" | "radio-group" => "div".to_string(),

            // Feedback
            "progress" => "progress".to_string(),
            "badge" => "span".to_string(),
            "spinner" => "div".to_string(),

            // Display
            "card" => "div".to_string(),
            "avatar" => "img".to_string(),
            "aspectratio" | "aspect-ratio" => "div".to_string(),

            // Media
            "image" => "img".to_string(),
            "img" => "img".to_string(),
            "icon" => "span".to_string(),

            // Utility
            "divider" => "hr".to_string(),
            "spacer" => "div".to_string(),

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

    /// Extract Tailwind classes from tag and props
    /// Returns (static_classes, dynamic_class_binding)
    fn extract_classes(&self, tag: &str, props: &HashMap<String, AuraPropValue>) -> (String, Option<String>) {
        let mut classes = Vec::new();
        let mut dynamic_binding: Option<String> = None;

        // In shadcn mode, skip default classes for components that have shadcn versions
        // (shadcn components have their own styling)
        let skip_defaults = self.is_shadcn() && self.shadcn_registry.has_component(tag);

        // Check if user has provided a class attribute
        let has_user_class = props.contains_key("class");

        // For semantic HTML elements, skip defaults if user provides their own class
        // These elements are typically fully custom-styled
        let semantic_elements = ["header", "nav", "main", "aside", "footer", "article"];
        let skip_semantic_defaults = has_user_class && semantic_elements.contains(&tag);

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
            match tag {
                // Layout
                "col" | "column" => classes.push(format!("flex flex-col {}", gap_class)),
                "row" => classes.push(format!("flex flex-row {}", gap_class)),
                "grid" => classes.push("grid".to_string()),
                "scroll" => classes.push("overflow-auto".to_string()),
                "container" => classes.push("max-w-7xl mx-auto".to_string()),
                "center" => classes.push("flex items-center justify-center".to_string()),

                // HTML5 semantic elements (only add defaults if user hasn't provided class)
                "header" => classes.push("w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60".to_string()),
                "nav" => classes.push("flex items-center gap-4".to_string()),
                "main" => classes.push("flex-1".to_string()),
                "aside" => classes.push("w-64 border-r bg-background".to_string()),
                "footer" => classes.push("w-full border-t bg-background".to_string()),
                "article" => classes.push("prose max-w-none".to_string()),

                // Typography
                "h1" => classes.push("text-4xl font-bold tracking-tight".to_string()),
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
                "label" => classes.push("text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70".to_string()),

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

        // Class prop
        if let Some(value) = props.get("class") {
            match value {
                AuraPropValue::Expr(AuraExpr::Literal(s)) => {
                    classes.push(s.clone());
                }
                AuraPropValue::ClassBinding(bindings) => {
                    // Generate dynamic class binding: { completed: todo.done, editing: todo.editing }
                    let binding_strs: Vec<String> = bindings.iter()
                        .map(|b| {
                            let cond = self.expr_to_js(&b.condition).unwrap_or_else(|_| "false".to_string());
                            format!("{}: {}", b.class_name, cond)
                        })
                        .collect();
                    dynamic_binding = Some(format!("{{ {} }}", binding_strs.join(", ")));
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

    /// Convert AuraExpr to JS value string
    fn expr_to_js(&self, expr: &AuraExpr) -> GenResult<String> {
        match expr {
            AuraExpr::Literal(s) => Ok(format!("'{}'", s)),
            AuraExpr::Int(n) => Ok(n.to_string()),
            AuraExpr::Float(n) => Ok(n.to_string()),
            AuraExpr::Bool(b) => Ok(b.to_string()),
            AuraExpr::StateRef(name) => {
                if self.state_names.contains(&name.to_string()) {
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
                let object_js = self.expr_to_js(object)?;
                let args_js: Vec<String> = args.iter()
                    .map(|a| self.expr_to_js(a))
                    .collect::<Result<Vec<_>, _>>()?;
                // Convert .len to .length for JavaScript
                let method_js = if method == "len" { "length" } else { method.as_str() };
                Ok(format!("{}.{}({})", object_js, method_js, args_js.join(", ")))
            }
            AuraExpr::Array(elems) => {
                let elems_js: Vec<String> = elems.iter()
                    .map(|e| self.expr_to_js(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("[{}]", elems_js.join(", ")))
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
        }
    }

    /// Convert AuraStmt to JS statement
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

    /// Convert prop value to HTML attribute value
    fn prop_to_attr_value(&self, value: &AuraPropValue) -> GenResult<String> {
        match value {
            AuraPropValue::Expr(expr) => {
                match expr {
                    AuraExpr::Literal(s) => Ok(format!("\"{}\"", s)),
                    AuraExpr::Int(n) => Ok(format!("\"{}\"", n)),
                    AuraExpr::Bool(b) => Ok(format!("\"{}\"", b)),
                    AuraExpr::StateRef(name) => Ok(format!("\"{{{{ {} }}}}\"", name)),
                    _ => Ok(format!("\"{{{{ {} }}}}\"", "value")),
                }
            }
            AuraPropValue::ClassBinding(_) => {
                // Class bindings are handled separately in extract_classes
                Ok("\"\"".to_string())
            }
        }
    }

    /// Convert prop value to text content (for rendering inside element)
    fn prop_to_text_content(&self, value: &AuraPropValue) -> GenResult<String> {
        match value {
            AuraPropValue::Expr(expr) => {
                match expr {
                    AuraExpr::Literal(s) => Ok(s.clone()),
                    AuraExpr::Int(n) => Ok(n.to_string()),
                    AuraExpr::Bool(b) => Ok(b.to_string()),
                    AuraExpr::StateRef(name) => Ok(format!("{{{{ {} }}}}", name)),
                    _ => Ok("value".to_string()),
                }
            }
            AuraPropValue::ClassBinding(_) => {
                Ok("".to_string())
            }
        }
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

        // Normalize tag for matching (kebab-case -> snake_case)
        let normalized_tag = tag.replace('-', "_");

        match normalized_tag.as_str() {
            // === Button ===
            "button" => {
                // Handle variant prop
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("variant=\"{}\"", variant));
                }
                // Handle size prop
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
                // Text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
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
            }

            // === Checkbox ===
            "checkbox" => {
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
            "selectitem" | "select-item" => {
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
            "selectvalue" | "select-value" => {
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
            }

            // === SelectTrigger ===
            "selecttrigger" | "select-trigger" => {
                // class
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === SelectLabel ===
            "selectlabel" | "select-label" => {
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
                if let Some(value) = props.get("type") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("variant=\"{}\"", variant));
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

            // === ScrollArea ===
            "scroll" => {
                // viewport class for styling
                if let Some(value) = props.get("class") {
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
            "alertdialog" | "alert-dialog" => {
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
                if let Some(value) = props.get("class") {
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
            "tabslist" | "tabs-list" => {
                // TabsList is a container - class handled by extract_classes
            }
            "tabstrigger" | "tabs-trigger" => {
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
            "tabscontent" | "tabs-content" => {
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

            // === AspectRatio ===
            "aspectratio" | "aspect-ratio" => {
                // ratio prop (e.g., 16/9 = 1.777)
                if let Some(value) = props.get("ratio") {
                    if let Some(ratio) = self.extract_float_value(value) {
                        attrs.push(format!(":ratio=\"{}\"", ratio));
                    }
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "thead" | "tbody" | "tr" => {
                // Table structure elements - minimal props
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "th" | "td" => {
                // Table cells
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "table_head" | "table_cell" => {
                // class
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
            }

            "sheet_content" => {
                // side: top, right, bottom, left
                if let Some(value) = props.get("side") {
                    let side = self.extract_string_value(value).unwrap_or("right");
                    attrs.push(format!("side=\"{}\"", side));
                }
                // class
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "sheet_header" | "sheet_footer" => {
                // class
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "breadcrumb_item" => {
                // class
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Alert Dialog ===
            "alert_dialog" => {
                // v-model:open for dialog state
                if let Some(value) = props.get("open") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:open=\"{}\"", model));
                    }
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
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "alert_dialog_header" | "alert_dialog_footer" => {
                // class
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "nav_menu_list" => {
                // class
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "sidebar_content" => {
                // class
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "sidebar_group" => {
                // class
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "sidebar_menu" => {
                // class
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "sidebar_menu_item" => {
                // class
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "sidebar_provider" => {
                // class
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "carousel_content" | "carousel_item" => {
                // class
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "carousel_prev" | "carousel_previous" | "carousel_next" => {
                // class
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
            }

            "drawer_content" => {
                // class
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            "drawer_header" | "drawer_footer" => {
                // class
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                // sibling-count
                if let Some(value) = props.get("sibling_count") {
                    if let Some(count) = self.extract_int_value(value) {
                        attrs.push(format!(":sibling-count=\"{}\"", count));
                    }
                }
            }

            "pagination_list" => {
                // class
                if let Some(value) = props.get("class") {
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

            // === Aspect Ratio ===
            "aspect_ratio" => {
                // ratio (default 16/9 = 1.777...)
                if let Some(value) = props.get("ratio") {
                    if let Some(ratio) = self.extract_int_value(value) {
                        attrs.push(format!(":ratio=\"{}\"", ratio));
                    }
                }
                // class
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Input Group ===
            "input_group" => {
                // class
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
                    let class = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("class=\"{}\"", class));
                }
            }

            // === Menubar ===
            "menubar" => {
                // class
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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
                if let Some(value) = props.get("class") {
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

            _ => {
                // Default handling for other components
            }
        }

        // Add event handlers
        for (event, aura_event) in events {
            let vue_event = self.shadcn_event_to_vue(tag, event);
            let handler_fn = self.handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
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

    /// Convert AutoUI event name to Vue event
    fn auto_event_to_vue(&self, event: &str) -> String {
        match event {
            "onclick" | "onClick" | "on_click" => "@click".to_string(),
            "oninput" | "onInput" => "@input".to_string(),
            "onchange" | "onChange" => "@change".to_string(),
            _ => format!("@{}", event.trim_start_matches("on")),
        }
    }

    /// Convert handler pattern to function name
    fn pattern_to_handler_name(&self, pattern: &str) -> String {
        // Check for dot prefix first (e.g., ".Inc")
        if pattern.starts_with('.') {
            format!("on{}", &pattern[1..])
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
            format!("on{}", &handler[1..])
        } else if let Some(variant) = handler.split("::").last() {
            // Handler like "Msg::Inc" -> "onInc"
            format!("on{}", variant)
        } else {
            format!("on{}", handler)
        }
    }

    /// Convert handler to Vue function call with parameters
    fn handler_to_function_call_with_params(&self, handler: &str, params: &[String]) -> String {
        let func_name = self.handler_to_function_call(handler);
        if params.is_empty() {
            func_name
        } else {
            format!("{}({})", func_name, params.join(", "))
        }
    }

    // ========================================================================
    // shadcn-vue Support Methods
    // ========================================================================

    /// Register a shadcn-vue component as used
    fn register_shadcn_component(&mut self, tag: &str) {
        if self.is_shadcn() {
            if let Some(component_name) = self.shadcn_registry.primary_component(tag) {
                self.shadcn_components_used.insert(component_name.to_string());
            }
        }
    }

    /// Generate shadcn-vue import statements
    fn generate_shadcn_imports(&self) -> String {
        if !self.is_shadcn() || self.shadcn_components_used.is_empty() {
            return String::new();
        }

        // Group components by their module path
        let mut module_imports: HashMap<&str, Vec<&str>> = HashMap::new();

        for component_name in &self.shadcn_components_used {
            // Find which module this component belongs to
            for (tag, (module, components)) in &self.shadcn_registry.components {
                if components.contains(&component_name.as_str()) {
                    module_imports.entry(module).or_default().push(component_name.as_str());
                    break;
                }
            }
        }

        let mut imports = Vec::new();
        for (module, components) in module_imports {
            let unique_components: HashSet<&str> = components.into_iter().collect();
            let mut sorted: Vec<&str> = unique_components.into_iter().collect();
            sorted.sort();
            imports.push(format!("import {{ {} }} from '{}'", sorted.join(", "), module));
        }

        imports.sort();
        imports.join("\n")
    }

    /// Get shadcn-vue component name for a tag
    fn shadcn_component_name(&self, tag: &str) -> Option<&'static str> {
        if self.is_shadcn() {
            self.shadcn_registry.primary_component(tag)
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
  "tsConfigPath": "./tsconfig.json",
  "tailwind": {
    "config": "tailwind.config.js",
    "css": "src/assets/index.css",
    "baseColor": "slate",
    "cssVariables": true
  },
  "framework": "vite",
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
    "vue-tsc": "^1.8.0",
    "tailwindcss": "^3.4.0",
    "tailwindcss-animate": "^1.0.7",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0"
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
            r#"import {{ createRouter, createWebHistory }} from 'vue-router'
import type {{ RouteRecordRaw }} from 'vue-router'

const routes: RouteRecordRaw[] = [
{}
]

const router = createRouter({{
  history: createWebHistory(),
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aura::{AuraMessage, AuraMsgVariant};
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
        };

        let mut gen = VueGenerator::new();
        let sfc = gen.generate(&widget).unwrap();

        // Plan 100: Default is now TypeScript, so check for lang="ts"
        assert!(sfc.contains(r#"<script setup lang="ts">"#));
        assert!(sfc.contains("import { ref } from 'vue'"));
        assert!(sfc.contains("const count = ref<number>(0)"));
        assert!(sfc.contains("<template>"));
        assert!(sfc.contains("<style scoped>"));
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
        assert_eq!(gen.pattern_to_handler_name(".Inc"), "onInc");
        assert_eq!(gen.pattern_to_handler_name("Dec"), "onDec");
    }

    #[test]
    fn test_shadcn_mode() {
        let mut gen = VueGenerator::new_shadcn();
        assert!(gen.is_shadcn());

        let gen = VueGenerator::new().with_mode(VueMode::Shadcn);
        assert!(gen.is_shadcn());

        let gen = VueGenerator::new();
        assert!(!gen.is_shadcn());
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
    fn test_generate_shadcn_attrs_button() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test button with text
        props.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Click me".to_string())));
        let (attrs, slot_content, _) = gen.generate_shadcn_attrs("button", &props, &events);

        assert!(slot_content.is_some());
        assert_eq!(slot_content.unwrap(), "Click me");
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

        // Test checkbox with v-model:checked
        props.insert("checked".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("done".to_string())));
        let (attrs, _, _) = gen.generate_shadcn_attrs("checkbox", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("v-model:checked=\"done\"")));
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
                params: vec![],
            },
            AuraRoute {
                path: "/about".to_string(),
                module: "about".to_string(),
                params: vec![],
            },
            AuraRoute {
                path: "/user/:id".to_string(),
                module: "user".to_string(),
                params: vec!["id".to_string()],
            },
        ];

        let output = VueGenerator::generate_router_file(&routes);

        // Check imports
        assert!(output.contains("import { createRouter, createWebHistory }"));

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
        assert!(output.contains("import { createRouter, createWebHistory }"));
        assert!(output.contains("const routes: RouteRecordRaw[] = ["));
        assert!(output.contains("export default router"));
    }
}
