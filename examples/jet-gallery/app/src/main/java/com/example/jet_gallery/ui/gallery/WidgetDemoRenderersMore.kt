@file:OptIn(ExperimentalMaterial3Api::class, ExperimentalLayoutApi::class)

package com.example.jet_gallery.ui.gallery

import android.widget.Toast
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Notifications
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch

data class DemoRecord(
    val title: String,
    val status: String,
    val owner: String,
)

@Composable
fun BreadcrumbDemo() {
    Row(verticalAlignment = Alignment.CenterVertically) {
        listOf("Docs", "Android", "Jet Gallery").forEachIndexed { index, item ->
            Text(item, modifier = Modifier.clickable { })
            if (index < 2) {
                Text(" / ", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}

@Composable
fun NavigationMenuDemo() {
    var selected by rememberSaveable { mutableStateOf("Components") }
    androidx.compose.foundation.layout.FlowRow(
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        listOf("Components", "Patterns", "Runtime", "Tests").forEach { item ->
            FilterChip(
                selected = selected == item,
                onClick = { selected = item },
                label = { Text(item) },
            )
        }
    }
}

@Composable
fun PaginationDemo() {
    var page by rememberSaveable { mutableIntStateOf(2) }
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        TextButton(onClick = { page = (page - 1).coerceAtLeast(1) }) { Text("Prev") }
        (1..5).forEach { index ->
            AssistChip(onClick = { page = index }, label = { Text(index.toString()) })
        }
        TextButton(onClick = { page = (page + 1).coerceAtMost(5) }) { Text("Next") }
    }
}

@Composable
fun SidebarDemo() {
    Row(horizontalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
        Surface(
            shape = RoundedCornerShape(24.dp),
            color = MaterialTheme.colorScheme.surfaceVariant,
            modifier = Modifier.width(180.dp),
        ) {
            Column(modifier = Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                listOf("Overview", "Forms", "Overlays", "Data").forEach { item ->
                    FilterChip(selected = item == "Forms", onClick = {}, label = { Text(item) })
                }
            }
        }
        Text("Sidebar is usually expressed as a rail or drawer on Android.", modifier = Modifier.weight(1f))
    }
}

@Composable
fun MenuBarDemo() {
    var expanded by rememberSaveable { mutableStateOf(false) }
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        TextButton(onClick = {}) { Text("File") }
        Box {
            TextButton(onClick = { expanded = true }) { Text("Edit") }
            DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
                listOf("Undo", "Redo", "Duplicate").forEach { item ->
                    DropdownMenuItem(text = { Text(item) }, onClick = { expanded = false })
                }
            }
        }
        TextButton(onClick = {}) { Text("View") }
    }
}

@Composable
fun DropdownMenuDemo() {
    var expanded by rememberSaveable { mutableStateOf(false) }
    Box {
        Button(onClick = { expanded = true }) { Text("Open menu") }
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            listOf("Rename", "Duplicate", "Archive").forEach { item ->
                DropdownMenuItem(text = { Text(item) }, onClick = { expanded = false })
            }
        }
    }
}

@Composable
fun NavLinkDemo() {
    var route by rememberSaveable { mutableStateOf("/button") }
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            listOf("/button", "/tabs", "/table").forEach { item ->
                Text(
                    text = item,
                    color = if (route == item) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface,
                    textDecoration = TextDecoration.Underline,
                    modifier = Modifier.clickable { route = item },
                )
            }
        }
        Text("Current route: $route")
    }
}

@Composable
fun DialogDemo() {
    var open by rememberSaveable { mutableStateOf(false) }
    Button(onClick = { open = true }) { Text("Open dialog") }
    if (open) {
        AlertDialog(
            onDismissRequest = { open = false },
            title = { Text("Compose dialog") },
            text = { Text("Dialogs are stateful overlays in Compose.") },
            confirmButton = { TextButton(onClick = { open = false }) { Text("OK") } },
        )
    }
}

@Composable
fun AlertDialogDemo() {
    var open by rememberSaveable { mutableStateOf(false) }
    OutlinedButton(onClick = { open = true }) { Text("Delete draft") }
    if (open) {
        AlertDialog(
            onDismissRequest = { open = false },
            title = { Text("Delete draft?") },
            text = { Text("This action cannot be undone.") },
            dismissButton = { TextButton(onClick = { open = false }) { Text("Cancel") } },
            confirmButton = { Button(onClick = { open = false }) { Text("Delete") } },
        )
    }
}

@Composable
fun SheetDemo() {
    var open by rememberSaveable { mutableStateOf(false) }
    Button(onClick = { open = true }) { Text("Show sheet") }
    if (open) {
        ModalBottomSheet(onDismissRequest = { open = false }) {
            Column(modifier = Modifier.padding(24.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text("Bottom sheet", style = MaterialTheme.typography.titleLarge)
                Text("Compose uses ModalBottomSheet for this pattern.")
                Button(onClick = { open = false }) { Text("Close") }
            }
        }
    }
}

@Composable
fun DrawerDemo() {
    val drawerState = rememberDrawerState(initialValue = DrawerValue.Closed)
    val scope = rememberCoroutineScope()
    ModalNavigationDrawer(
        drawerState = drawerState,
        drawerContent = {
            ModalDrawerSheet {
                Text("Jet Gallery", modifier = Modifier.padding(16.dp), style = MaterialTheme.typography.titleLarge)
                listOf("Overview", "Components", "Patterns").forEach {
                    Text(it, modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp))
                }
            }
        },
    ) {
        OutlinedButton(onClick = { scope.launch { drawerState.open() } }) {
            Text("Open drawer")
        }
    }
}

@Composable
fun PopoverDemo() {
    var open by rememberSaveable { mutableStateOf(false) }
    Box {
        OutlinedButton(onClick = { open = true }) { Text("Show popover") }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            DropdownMenuItem(text = { Text("Popover content in Android is often a menu or popup.") }, onClick = { open = false })
        }
    }
}

@Composable
fun TooltipDemo() {
    var visible by rememberSaveable { mutableStateOf(false) }
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        IconButton(onClick = { visible = !visible }) {
            Icon(Icons.Default.Warning, contentDescription = "Help")
        }
        AnimatedVisibility(visible) {
            Surface(shape = RoundedCornerShape(16.dp), tonalElevation = 4.dp) {
                Text("Helpful contextual text.", modifier = Modifier.padding(12.dp))
            }
        }
    }
}

@Composable
fun HoverCardDemo() {
    var visible by rememberSaveable { mutableStateOf(false) }
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        OutlinedButton(onClick = { visible = !visible }) { Text("Peek preview") }
        AnimatedVisibility(visible) {
            ElevatedCard {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text("HoverCard approximation", style = MaterialTheme.typography.titleMedium)
                    Text("On touch devices this is usually a tap-triggered preview card.")
                }
            }
        }
    }
}

@Composable
fun ContextMenuDemo() {
    var expanded by rememberSaveable { mutableStateOf(false) }
    Box {
        IconButton(onClick = { expanded = true }) {
            Icon(Icons.Default.MoreVert, contentDescription = "More actions")
        }
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            listOf("Share", "Duplicate", "Delete").forEach { action ->
                DropdownMenuItem(text = { Text(action) }, onClick = { expanded = false })
            }
        }
    }
}

@Composable
fun AlertWidgetDemo() {
    Surface(shape = RoundedCornerShape(20.dp), color = MaterialTheme.colorScheme.errorContainer) {
        Row(
            modifier = Modifier.padding(16.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(Icons.Default.Warning, contentDescription = null)
            Column {
                Text("Build warning", fontWeight = FontWeight.SemiBold)
                Text("The generator still needs a composite strategy for hovercard.")
            }
        }
    }
}

@Composable
fun ToastWidgetDemo() {
    val context = LocalContext.current
    Button(onClick = { Toast.makeText(context, "Jet Gallery toast", Toast.LENGTH_SHORT).show() }) {
        Text("Show toast")
    }
}

@Composable
fun ProgressDemo() {
    var progress by rememberSaveable { mutableFloatStateOf(0.64f) }
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        LinearProgressIndicator(progress = { progress }, modifier = Modifier.fillMaxWidth())
        androidx.compose.material3.CircularProgressIndicator(progress = { progress })
        OutlinedButton(onClick = { progress = if (progress > 0.9f) 0.15f else progress + 0.15f }) { Text("Advance") }
    }
}

@Composable
fun SonnerDemo() {
    val hostState = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        SnackbarHost(hostState = hostState)
        Button(onClick = { scope.launch { hostState.showSnackbar("Widget saved to favorites") } }) {
            Text("Show snackbar")
        }
    }
}

@Composable
fun TableDemo() {
    val rows = remember {
        listOf(
            DemoRecord("Button", "Native", "Material3"),
            DemoRecord("Accordion", "Composite", "Custom"),
            DemoRecord("Swiper", "Composite", "Pager"),
        )
    }
    TableSurface(rows)
}

@Composable
fun DataTableDemo() {
    var activeFilter by rememberSaveable { mutableStateOf("All") }
    var ascending by rememberSaveable { mutableStateOf(true) }
    val rows = remember {
        listOf(
            DemoRecord("Accordion", "Composite", "Layout"),
            DemoRecord("Button", "Native", "Form"),
            DemoRecord("Table", "Composite", "Data"),
            DemoRecord("Tabs", "Native", "Navigation"),
        )
    }
    val filteredRows = rows.filter { activeFilter == "All" || it.status == activeFilter }
        .let { filtered ->
            val sorted = filtered.sortedBy { it.title }
            if (ascending) sorted else sorted.reversed()
        }

    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            listOf("All", "Native", "Composite").forEach { chip ->
                FilterChip(selected = activeFilter == chip, onClick = { activeFilter = chip }, label = { Text(chip) })
            }
            Spacer(modifier = Modifier.weight(1f))
            OutlinedButton(onClick = { ascending = !ascending }) { Text(if (ascending) "A→Z" else "Z→A") }
        }
        TableSurface(filteredRows)
    }
}

@Composable
fun CalendarDemo() {
    var selectedDay by rememberSaveable { mutableIntStateOf(14) }
    val days = (1..30).map(Int::toString)
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Text("April 2026", style = MaterialTheme.typography.titleLarge)
        LazyVerticalGrid(
            columns = GridCells.Fixed(7),
            modifier = Modifier.height(220.dp),
            userScrollEnabled = false,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            items(days) { day ->
                val dayValue = day.toInt()
                Surface(
                    shape = CircleShape,
                    color = if (selectedDay == dayValue) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceVariant,
                    modifier = Modifier
                        .size(40.dp)
                        .clickable { selectedDay = dayValue },
                ) {
                    Box(contentAlignment = Alignment.Center, modifier = Modifier.fillMaxWidth()) {
                        Text(day, color = if (selectedDay == dayValue) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurface)
                    }
                }
            }
        }
    }
}

@Composable
fun GridDemo() {
    val items = (1..6).map { "Cell $it" }
    LazyVerticalGrid(
        columns = GridCells.Fixed(2),
        modifier = Modifier.height(220.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
        userScrollEnabled = false,
    ) {
        items(items) { label ->
            SampleCard(label, Modifier.fillMaxWidth())
        }
    }
}

@Composable
fun GridItemDemo() {
    Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        SampleCard("Grid item", Modifier.weight(1f))
        Text("Grid items are child cells inside a LazyVerticalGrid.", modifier = Modifier.weight(1f))
    }
}

@Composable
fun ListDemo() {
    LazyColumn(
        modifier = Modifier.height(220.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
        contentPadding = PaddingValues(vertical = 4.dp),
    ) {
        items((1..5).map { "List row $it" }) { label ->
            SampleCard(label, Modifier.fillMaxWidth())
        }
    }
}

@Composable
fun ListItemDemo() {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        ListItem(
            headlineContent = { Text("Jet Gallery") },
            supportingContent = { Text("Reference app for a2jet targets") },
            leadingContent = { Icon(Icons.Default.Notifications, contentDescription = null) },
        )
        HorizontalDivider()
        ListItem(
            headlineContent = { Text("Composite widgets") },
            supportingContent = { Text("Sidebar, Sheet, DataTable, Swiper") },
        )
    }
}

@Composable
fun TableSurface(rows: List<DemoRecord>) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .border(1.dp, MaterialTheme.colorScheme.outlineVariant, RoundedCornerShape(20.dp)),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(MaterialTheme.colorScheme.surfaceVariant)
                .padding(12.dp),
        ) {
            Text("Widget", modifier = Modifier.weight(1f), fontWeight = FontWeight.Bold)
            Text("Tier", modifier = Modifier.weight(1f), fontWeight = FontWeight.Bold)
            Text("Area", modifier = Modifier.weight(1f), fontWeight = FontWeight.Bold)
        }
        rows.forEach { row ->
            HorizontalDivider()
            Row(modifier = Modifier.fillMaxWidth().padding(12.dp)) {
                Text(row.title, modifier = Modifier.weight(1f))
                Text(row.status, modifier = Modifier.weight(1f))
                Text(row.owner, modifier = Modifier.weight(1f))
            }
        }
    }
}
