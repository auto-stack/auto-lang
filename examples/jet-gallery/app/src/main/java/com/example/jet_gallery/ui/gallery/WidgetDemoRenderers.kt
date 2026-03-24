@file:OptIn(ExperimentalFoundationApi::class)

package com.example.jet_gallery.ui.gallery

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.res.painterResource
import coil.compose.AsyncImage
import com.example.jet_gallery.R

@Composable
fun WidgetLiveDemo(widget: WidgetDemo) {
    when (widget.id) {
        "col" -> ColumnDemo()
        "row" -> RowDemo()
        "center" -> CenterDemo()
        "card" -> CardDemo()
        "scrollarea" -> ScrollAreaDemo()
        "aspectratio" -> AspectRatioDemo()
        "collapsible" -> CollapsibleDemo()
        "accordion" -> AccordionDemo()
        "button" -> ButtonDemo()
        "input" -> InputDemo()
        "checkbox" -> CheckboxDemo()
        "switch" -> SwitchDemo()
        "select" -> SelectDemo()
        "slider" -> SliderDemo()
        "radiogroup" -> RadioGroupDemo()
        "textarea" -> TextareaDemo()
        "form" -> FormDemo()
        "text" -> TextDemo()
        "image" -> ImageDemo()
        "badge" -> BadgeDemo()
        "avatar" -> AvatarDemo()
        "separator" -> SeparatorDemo()
        "skeleton" -> SkeletonDemo()
        "swiper" -> SwiperDemo()
        "tabs" -> TabsDemo()
        "breadcrumb" -> BreadcrumbDemo()
        "navigationmenu" -> NavigationMenuDemo()
        "pagination" -> PaginationDemo()
        "sidebar" -> SidebarDemo()
        "menubar" -> MenuBarDemo()
        "dropdownmenu" -> DropdownMenuDemo()
        "navlink" -> NavLinkDemo()
        "dialog" -> DialogDemo()
        "alertdialog" -> AlertDialogDemo()
        "sheet" -> SheetDemo()
        "drawer" -> DrawerDemo()
        "popover" -> PopoverDemo()
        "tooltip" -> TooltipDemo()
        "hovercard" -> HoverCardDemo()
        "contextmenu" -> ContextMenuDemo()
        "alert" -> AlertWidgetDemo()
        "toast" -> ToastWidgetDemo()
        "progress" -> ProgressDemo()
        "sonner" -> SonnerDemo()
        "table" -> TableDemo()
        "datatable" -> DataTableDemo()
        "calendar" -> CalendarDemo()
        "grid" -> GridDemo()
        "griditem" -> GridItemDemo()
        "list" -> ListDemo()
        "listitem" -> ListItemDemo()
    }
}

@Composable
fun WidgetVariantDemo(widget: WidgetDemo) {
    when (widget.id) {
        "button" -> Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Button(onClick = {}) { Text("Primary") }
            OutlinedButton(onClick = {}) { Text("Outlined") }
            TextButton(onClick = {}) { Text("Ghost") }
        }
        "input", "textarea", "form" -> Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
            OutlinedTextField(value = "Filled", onValueChange = {}, label = { Text("Enabled") })
            OutlinedTextField(value = "", onValueChange = {}, label = { Text("Disabled") }, enabled = false)
        }
        "checkbox", "switch", "radiogroup" -> FlowTagRow(listOf("checked", "unchecked", "disabled"))
        "slider", "progress" -> Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
            LinearProgressIndicator(progress = { 0.3f }, modifier = Modifier.fillMaxWidth())
            LinearProgressIndicator(progress = { 0.85f }, modifier = Modifier.fillMaxWidth())
        }
        "tabs", "dropdownmenu", "navigationmenu", "sidebar", "menubar" -> FlowTagRow(listOf("selected", "collapsed", "disabled"))
        "dialog", "alertdialog", "sheet", "drawer", "popover", "tooltip", "hovercard", "contextmenu" ->
            Text("Overlay variants focus on trigger, state, and dismiss behavior.", style = MaterialTheme.typography.bodyMedium)
        "table", "datatable", "calendar", "grid", "list", "listitem", "griditem" ->
            Text("Data widgets vary by density, selected row/item state, and scroll behavior.", style = MaterialTheme.typography.bodyMedium)
        else -> FlowTagRow(listOf("default", "compact", "themed"))
    }
}

@Composable
fun SampleCard(label: String, modifier: Modifier = Modifier) {
    Surface(
        modifier = modifier,
        shape = RoundedCornerShape(18.dp),
        color = MaterialTheme.colorScheme.surfaceVariant,
    ) {
        Box(contentAlignment = Alignment.Center, modifier = Modifier.padding(16.dp)) {
            Text(label, fontWeight = FontWeight.SemiBold)
        }
    }
}

@Composable
private fun ColumnDemo() {
    Column(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
        SampleCard("Header", Modifier.fillMaxWidth())
        SampleCard("Content", Modifier.fillMaxWidth())
        SampleCard("Footer", Modifier.fillMaxWidth())
    }
}

@Composable
private fun RowDemo() {
    Row(horizontalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
        SampleCard("Nav", Modifier.weight(1f))
        SampleCard("Feed", Modifier.weight(2f))
        SampleCard("Side", Modifier.weight(1f))
    }
}

@Composable
private fun CenterDemo() {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(180.dp)
            .clip(RoundedCornerShape(24.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant),
        contentAlignment = Alignment.Center,
    ) {
        AssistChip(onClick = {}, label = { Text("Centered content") })
    }
}

@Composable
private fun CardDemo() {
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Card { Text("Standard Card", modifier = Modifier.padding(16.dp)) }
        ElevatedCard { Text("Elevated Card", modifier = Modifier.padding(16.dp)) }
        OutlinedCard { Text("Outlined Card", modifier = Modifier.padding(16.dp)) }
    }
}

@Composable
private fun ScrollAreaDemo() {
    val scrollState = rememberScrollState()
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .height(180.dp)
            .clip(RoundedCornerShape(24.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant)
            .verticalScroll(scrollState)
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        repeat(8) {
            SampleCard("Scrollable item ${it + 1}", Modifier.fillMaxWidth())
        }
    }
}

@Composable
private fun AspectRatioDemo() {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .aspectRatio(16f / 9f)
            .clip(RoundedCornerShape(24.dp))
            .background(MaterialTheme.colorScheme.primaryContainer),
        contentAlignment = Alignment.Center,
    ) {
        Text("16:9 media frame", style = MaterialTheme.typography.titleMedium)
    }
}

@Composable
private fun CollapsibleDemo() {
    var expanded by rememberSaveable { mutableStateOf(false) }
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        OutlinedButton(onClick = { expanded = !expanded }) {
            Text(if (expanded) "Hide details" else "Show details")
        }
        AnimatedVisibility(expanded) {
            ElevatedCard {
                Text(
                    "Collapsible content can be composed with state and AnimatedVisibility in Compose.",
                    modifier = Modifier.padding(16.dp),
                )
            }
        }
    }
}

@Composable
private fun AccordionDemo() {
    val expandedItems = remember { mutableStateListOf(0) }
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        listOf("Architecture", "Styling", "State").forEachIndexed { index, title ->
            OutlinedCard {
                Column(modifier = Modifier.fillMaxWidth()) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clickable {
                                if (expandedItems.contains(index)) expandedItems.remove(index) else expandedItems.add(index)
                            }
                            .padding(16.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(title, style = MaterialTheme.typography.titleMedium)
                        Text(if (expandedItems.contains(index)) "−" else "+", style = MaterialTheme.typography.titleMedium)
                    }
                    AnimatedVisibility(expandedItems.contains(index)) {
                        Text("Accordion item body for $title.", modifier = Modifier.padding(start = 16.dp, end = 16.dp, bottom = 16.dp))
                    }
                }
            }
        }
    }
}

@Composable
private fun ButtonDemo() {
    var clicks by rememberSaveable { mutableIntStateOf(0) }
    Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
        Button(onClick = { clicks += 1 }) { Text("Tap me") }
        OutlinedButton(onClick = {}) { Text("Secondary") }
        Text("Clicks: $clicks")
    }
}

@Composable
private fun InputDemo() {
    var value by rememberSaveable { mutableStateOf("Ada Lovelace") }
    OutlinedTextField(
        value = value,
        onValueChange = { value = it },
        modifier = Modifier.fillMaxWidth(),
        label = { Text("Name") },
        supportingText = { Text("Single-line material input.") },
    )
}

@Composable
private fun CheckboxDemo() {
    var checked by rememberSaveable { mutableStateOf(true) }
    Row(verticalAlignment = Alignment.CenterVertically) {
        Checkbox(checked = checked, onCheckedChange = { checked = it })
        Text("Enable experimental widgets")
    }
}

@Composable
private fun SwitchDemo() {
    var enabled by rememberSaveable { mutableStateOf(false) }
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Column {
            Text("Adaptive tablet mode")
            Text("Use a navigation rail when the width is wide enough.", style = MaterialTheme.typography.bodyMedium)
        }
        Switch(checked = enabled, onCheckedChange = { enabled = it })
    }
}

@Composable
private fun SelectDemo() {
    var expanded by rememberSaveable { mutableStateOf(false) }
    var selected by rememberSaveable { mutableStateOf("Stable") }
    Box {
        OutlinedButton(onClick = { expanded = true }) {
            Text("Release channel: $selected")
        }
        androidx.compose.material3.DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            listOf("Stable", "Beta", "Canary").forEach { item ->
                androidx.compose.material3.DropdownMenuItem(
                    text = { Text(item) },
                    onClick = {
                        selected = item
                        expanded = false
                    },
                )
            }
        }
    }
}

@Composable
private fun SliderDemo() {
    var value by rememberSaveable { mutableFloatStateOf(0.42f) }
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text("Density: ${(value * 100).toInt()}%")
        androidx.compose.material3.Slider(value = value, onValueChange = { value = it })
    }
}

@Composable
private fun RadioGroupDemo() {
    var selected by rememberSaveable { mutableStateOf("Material 3") }
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        listOf("Material 3", "Custom", "Legacy").forEach { option ->
            Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                androidx.compose.material3.RadioButton(selected = selected == option, onClick = { selected = option })
                Text(option)
            }
        }
    }
}

@Composable
private fun TextareaDemo() {
    var notes by rememberSaveable {
        mutableStateOf("Jet Gallery gives us a native Android reference for composite widgets.")
    }
    OutlinedTextField(
        value = notes,
        onValueChange = { notes = it },
        modifier = Modifier.fillMaxWidth(),
        label = { Text("Notes") },
        minLines = 4,
    )
}

@Composable
private fun FormDemo() {
    var name by rememberSaveable { mutableStateOf("") }
    var email by rememberSaveable { mutableStateOf("") }
    var submitted by rememberSaveable { mutableStateOf(false) }

    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        OutlinedTextField(value = name, onValueChange = { name = it }, modifier = Modifier.fillMaxWidth(), label = { Text("Name") })
        OutlinedTextField(value = email, onValueChange = { email = it }, modifier = Modifier.fillMaxWidth(), label = { Text("Email") })
        Button(onClick = { submitted = true }) { Text("Submit") }
        if (submitted) {
            Text(
                if (name.isNotBlank() && email.contains("@")) "Looks valid." else "Show validation messages here.",
                color = if (name.isNotBlank() && email.contains("@")) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.error,
            )
        }
    }
}

@Composable
private fun TextDemo() {
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Text("Headline Large", style = MaterialTheme.typography.headlineLarge)
        Text("Title Large", style = MaterialTheme.typography.titleLarge)
        Text("Body Large for descriptive copy.", style = MaterialTheme.typography.bodyLarge)
        Text("Label Large for metadata.", style = MaterialTheme.typography.labelLarge)
    }
}

@Composable
private fun ImageDemo() {
    AsyncImage(
        model = R.drawable.ic_launcher_foreground,
        contentDescription = "Jet Gallery image sample",
        modifier = Modifier
            .fillMaxWidth()
            .height(180.dp)
            .clip(RoundedCornerShape(24.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant),
    )
}

@Composable
private fun BadgeDemo() {
    Row(horizontalArrangement = Arrangement.spacedBy(24.dp), verticalAlignment = Alignment.CenterVertically) {
        BadgedBox(badge = { Badge { Text("8") } }) {
            Icon(
                painter = painterResource(R.drawable.ic_launcher_foreground),
                contentDescription = null,
                modifier = Modifier.size(28.dp),
            )
        }
        Badge { Text("Beta") }
    }
}

@Composable
private fun AvatarDemo() {
    Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
        AsyncImage(
            model = R.drawable.ic_launcher_foreground,
            contentDescription = "Avatar",
            modifier = Modifier
                .size(56.dp)
                .clip(CircleShape)
                .background(MaterialTheme.colorScheme.surfaceVariant),
        )
        Surface(shape = CircleShape, color = MaterialTheme.colorScheme.primaryContainer) {
            Box(modifier = Modifier.size(56.dp), contentAlignment = Alignment.Center) {
                Text("JG", fontWeight = FontWeight.Bold)
            }
        }
    }
}

@Composable
private fun SeparatorDemo() {
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Text("Above the divider")
        HorizontalDivider()
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(48.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("Left", modifier = Modifier.weight(1f))
            androidx.compose.material3.VerticalDivider(modifier = Modifier.height(48.dp))
            Text("Right", modifier = Modifier.weight(1f))
        }
    }
}

@Composable
private fun SkeletonDemo() {
    val alpha by rememberInfiniteTransition(label = "skeleton").animateFloat(
        initialValue = 0.4f,
        targetValue = 0.9f,
        animationSpec = infiniteRepeatable(animation = tween(900), repeatMode = RepeatMode.Reverse),
        label = "alpha",
    )
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(22.dp)
                .clip(RoundedCornerShape(999.dp))
                .background(MaterialTheme.colorScheme.surfaceVariant)
                .alpha(alpha),
        )
        Box(
            modifier = Modifier
                .fillMaxWidth(0.7f)
                .height(18.dp)
                .clip(RoundedCornerShape(999.dp))
                .background(MaterialTheme.colorScheme.surfaceVariant)
                .alpha(alpha),
        )
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(120.dp)
                .clip(RoundedCornerShape(24.dp))
                .background(MaterialTheme.colorScheme.surfaceVariant)
                .alpha(alpha),
        )
    }
}

@Composable
private fun SwiperDemo() {
    val pagerItems = listOf("Overview", "Patterns", "Limitations")
    val pagerState = rememberPagerState(pageCount = { pagerItems.size })
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        HorizontalPager(
            state = pagerState,
            modifier = Modifier
                .fillMaxWidth()
                .height(180.dp),
        ) { page ->
            Surface(
                shape = RoundedCornerShape(24.dp),
                color = if (page % 2 == 0) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.secondaryContainer,
                modifier = Modifier
                    .fillMaxSize()
                    .padding(horizontal = 4.dp),
            ) {
                Box(contentAlignment = Alignment.Center, modifier = Modifier.fillMaxSize()) {
                    Text(pagerItems[page], style = MaterialTheme.typography.titleLarge)
                }
            }
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            repeat(pagerItems.size) { index ->
                Box(
                    modifier = Modifier
                        .size(if (pagerState.currentPage == index) 10.dp else 8.dp)
                        .clip(CircleShape)
                        .background(
                            if (pagerState.currentPage == index) MaterialTheme.colorScheme.primary
                            else MaterialTheme.colorScheme.outlineVariant,
                        ),
                )
            }
        }
    }
}

@Composable
private fun TabsDemo() {
    var selected by rememberSaveable { mutableIntStateOf(0) }
    val labels = listOf("Preview", "Code", "Notes")
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        TabRow(selectedTabIndex = selected) {
            labels.forEachIndexed { index, label ->
                Tab(selected = selected == index, onClick = { selected = index }, text = { Text(label) })
            }
        }
        Text("Active tab: ${labels[selected]}")
    }
}
