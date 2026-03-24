@file:OptIn(ExperimentalLayoutApi::class, ExperimentalMaterial3Api::class)

package com.example.jet_gallery.ui.gallery

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Apps
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Dashboard
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Menu
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp

private val sectionIcons = mapOf(
    AppSection.Home to Icons.Default.Home,
    AppSection.Foundation to Icons.Default.Dashboard,
    AppSection.Input to Icons.Default.Edit,
    AppSection.Structure to Icons.Default.Menu,
    AppSection.Data to Icons.Default.Apps,
)

@Composable
fun JetGalleryApp(windowWidthDpOverride: Int? = null) {
    val widthDp = windowWidthDpOverride ?: LocalConfiguration.current.screenWidthDp
    val isTablet = widthDp >= 840
    val initialWidget = WidgetDemoRegistry.firstForSection(AppSection.Foundation)?.id ?: WidgetDemoRegistry.demos.first().id

    var currentSection by rememberSaveable { mutableStateOf(AppSection.Home) }
    var selectedWidgetId by rememberSaveable { mutableStateOf(initialWidget) }
    var phoneShowingDetail by rememberSaveable { mutableStateOf(false) }

    val selectedWidget = WidgetDemoRegistry.require(selectedWidgetId)

    fun openSection(section: AppSection) {
        currentSection = section
        phoneShowingDetail = false
        val first = WidgetDemoRegistry.firstForSection(section)
        if (first != null && selectedWidget.section != section) {
            selectedWidgetId = first.id
        }
    }

    fun openWidget(widget: WidgetDemo) {
        selectedWidgetId = widget.id
        phoneShowingDetail = true
    }

    Scaffold(
        contentWindowInsets = WindowInsets.statusBars,
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        when {
                            currentSection == AppSection.Home -> "Jet Gallery"
                            !isTablet && phoneShowingDetail -> selectedWidget.title
                            else -> currentSection.title
                        },
                    )
                },
                navigationIcon = {
                    if (!isTablet && phoneShowingDetail) {
                        IconButton(onClick = { phoneShowingDetail = false }) {
                            Icon(Icons.Default.ArrowBack, contentDescription = "Back")
                        }
                    }
                },
                actions = {
                    if (currentSection != AppSection.Home && !phoneShowingDetail) {
                        Text(
                            text = "${WidgetDemoRegistry.groupedForSection(currentSection).values.flatten().size} demos",
                            style = MaterialTheme.typography.labelLarge,
                            color = MaterialTheme.colorScheme.primary,
                            modifier = Modifier.padding(end = 16.dp),
                        )
                    }
                },
            )
        },
        bottomBar = {
            if (!isTablet) {
                NavigationBar(
                    modifier = Modifier.testTag("bottom-nav"),
                    windowInsets = WindowInsets.navigationBars,
                ) {
                    AppSection.entries.forEach { section ->
                        GallerySectionItem(
                            section = section,
                            selected = currentSection == section,
                            onClick = { openSection(section) },
                        )
                    }
                }
            }
        },
    ) { innerPadding ->
        if (isTablet) {
            TabletGalleryContent(
                currentSection = currentSection,
                selectedWidget = selectedWidget,
                onSectionSelected = ::openSection,
                onWidgetSelected = { selectedWidgetId = it.id },
                modifier = Modifier.padding(innerPadding),
            )
        } else {
            PhoneGalleryContent(
                currentSection = currentSection,
                selectedWidget = selectedWidget,
                phoneShowingDetail = phoneShowingDetail,
                onSectionSelected = ::openSection,
                onWidgetSelected = ::openWidget,
                modifier = Modifier.padding(innerPadding),
            )
        }
    }
}

@Composable
private fun RowScope.GallerySectionItem(
    section: AppSection,
    selected: Boolean,
    onClick: () -> Unit,
) {
    val contentColor = if (selected) {
        MaterialTheme.colorScheme.primary
    } else {
        MaterialTheme.colorScheme.onSurfaceVariant
    }

    Surface(
        onClick = onClick,
        color = Color.Transparent,
        modifier = Modifier.weight(1f),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(vertical = 10.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            Icon(
                imageVector = sectionIcons.getValue(section),
                contentDescription = section.title,
                tint = contentColor,
            )
            Text(
                text = section.title,
                style = MaterialTheme.typography.labelMedium,
                color = contentColor,
            )
        }
    }
}

@Composable
private fun PhoneGalleryContent(
    currentSection: AppSection,
    selectedWidget: WidgetDemo,
    phoneShowingDetail: Boolean,
    onSectionSelected: (AppSection) -> Unit,
    onWidgetSelected: (WidgetDemo) -> Unit,
    modifier: Modifier = Modifier,
) {
    when {
        currentSection == AppSection.Home -> HomeOverview(onSectionSelected = onSectionSelected, modifier = modifier.fillMaxSize())
        phoneShowingDetail -> WidgetDetailScreen(widget = selectedWidget, modifier = modifier.fillMaxSize())
        else -> GallerySectionList(
            section = currentSection,
            selectedWidgetId = selectedWidget.id,
            onWidgetSelected = onWidgetSelected,
            modifier = modifier.fillMaxSize(),
        )
    }
}

@Composable
private fun TabletGalleryContent(
    currentSection: AppSection,
    selectedWidget: WidgetDemo,
    onSectionSelected: (AppSection) -> Unit,
    onWidgetSelected: (WidgetDemo) -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(modifier = modifier.fillMaxSize()) {
        NavigationRail(
            modifier = Modifier
                .fillMaxHeight()
                .testTag("nav-rail"),
        ) {
            Spacer(modifier = Modifier.height(12.dp))
            AppSection.entries.forEach { section ->
                NavigationRailItem(
                    selected = currentSection == section,
                    onClick = { onSectionSelected(section) },
                    icon = { Icon(sectionIcons.getValue(section), contentDescription = section.title) },
                    label = { Text(section.title) },
                )
            }
        }
        VerticalDivider()
        if (currentSection == AppSection.Home) {
            HomeOverview(
                onSectionSelected = onSectionSelected,
                modifier = Modifier
                    .fillMaxSize()
                    .padding(horizontal = 20.dp),
            )
        } else {
            Row(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(horizontal = 16.dp, vertical = 12.dp),
                horizontalArrangement = Arrangement.spacedBy(16.dp),
            ) {
                Surface(
                    shape = RoundedCornerShape(28.dp),
                    tonalElevation = 2.dp,
                    modifier = Modifier
                        .weight(0.85f)
                        .fillMaxHeight(),
                ) {
                    GallerySectionList(
                        section = currentSection,
                        selectedWidgetId = selectedWidget.id,
                        onWidgetSelected = onWidgetSelected,
                        modifier = Modifier.fillMaxSize(),
                    )
                }
                Surface(
                    shape = RoundedCornerShape(28.dp),
                    tonalElevation = 2.dp,
                    modifier = Modifier
                        .weight(1.15f)
                        .fillMaxHeight(),
                ) {
                    WidgetDetailScreen(widget = selectedWidget, modifier = Modifier.fillMaxSize())
                }
            }
        }
    }
}

@Composable
private fun HomeOverview(
    onSectionSelected: (AppSection) -> Unit,
    modifier: Modifier = Modifier,
) {
    val counts = WidgetCategory.entries.associateWith { category ->
        WidgetDemoRegistry.demos.count { it.category == category }
    }

    LazyColumn(
        modifier = modifier,
        contentPadding = PaddingValues(20.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        item {
            ElevatedCard(
                shape = RoundedCornerShape(32.dp),
                colors = androidx.compose.material3.CardDefaults.elevatedCardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceVariant,
                ),
            ) {
                androidx.compose.foundation.layout.Column(
                    modifier = Modifier.padding(24.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    Text("Compose reference gallery for stdlib AURA widgets", style = MaterialTheme.typography.headlineMedium)
                    Text(
                        "This app serves as a native Android reference for what direct and composite a2jet widget mappings should look like.",
                        style = MaterialTheme.typography.bodyLarge,
                    )
                    FlowTagRow(listOf("51 widget demos", "Adaptive shell", "Reference target for a2jet"))
                }
            }
        }
        item {
            Text("Sections", style = MaterialTheme.typography.titleLarge)
        }
        items(AppSection.entries.filter { it != AppSection.Home }) { section ->
            ElevatedCard(
                onClick = { onSectionSelected(section) },
                shape = RoundedCornerShape(24.dp),
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(20.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    androidx.compose.foundation.layout.Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text(section.title, style = MaterialTheme.typography.titleLarge)
                        Text(
                            WidgetCategory.entries.filter { it.section == section }
                                .joinToString(" • ") { category -> "${category.title}: ${counts.getValue(category)}" },
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    Icon(Icons.Default.ChevronRight, contentDescription = null)
                }
            }
        }
        item {
            ElevatedCard(shape = RoundedCornerShape(24.dp)) {
                androidx.compose.foundation.layout.Column(
                    modifier = Modifier.padding(20.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Text("Coverage policy", style = MaterialTheme.typography.titleMedium)
                    Text(
                        "The gallery includes stdlib widgets with a real or reasonable Compose equivalent and omits web-only gallery pages like command, datepicker, and toggle.",
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
        }
    }
}

@Composable
private fun GallerySectionList(
    section: AppSection,
    selectedWidgetId: String,
    onWidgetSelected: (WidgetDemo) -> Unit,
    modifier: Modifier = Modifier,
) {
    val grouped = WidgetDemoRegistry.groupedForSection(section)
    LazyColumn(
        modifier = modifier,
        contentPadding = PaddingValues(20.dp),
        verticalArrangement = Arrangement.spacedBy(18.dp),
    ) {
        grouped.forEach { (category, widgets) ->
            item {
                androidx.compose.foundation.layout.Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    Text(category.title, style = MaterialTheme.typography.titleLarge)
                    widgets.forEach { widget ->
                        WidgetListCard(
                            widget = widget,
                            selected = widget.id == selectedWidgetId,
                            onClick = { onWidgetSelected(widget) },
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun WidgetListCard(
    widget: WidgetDemo,
    selected: Boolean,
    onClick: () -> Unit,
) {
    OutlinedCard(
        onClick = onClick,
        shape = RoundedCornerShape(20.dp),
        border = androidx.compose.foundation.BorderStroke(
            width = if (selected) 2.dp else 1.dp,
            color = if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outlineVariant,
        ),
    ) {
        androidx.compose.foundation.layout.Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(widget.title, style = MaterialTheme.typography.titleMedium)
                MetaChip(widget.supportTier.title)
            }
            Text(widget.description, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
            Text(widget.composeTarget, style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.primary)
        }
    }
}

@Composable
fun WidgetDetailScreen(
    widget: WidgetDemo,
    modifier: Modifier = Modifier,
) {
    LazyColumn(
        modifier = modifier,
        contentPadding = PaddingValues(20.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        item {
            androidx.compose.foundation.layout.Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                Text(
                    text = widget.title,
                    style = MaterialTheme.typography.headlineMedium,
                    modifier = Modifier.testTag("gallery-detail-title"),
                )
                Text(widget.description, style = MaterialTheme.typography.bodyLarge)
                FlowTagRow(listOf(widget.category.title, widget.supportTier.title, widget.stdlibPath))
            }
        }
        item {
            PanelCard("Target Compose Pattern") {
                Text(widget.composeTarget, style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.primary)
            }
        }
        item {
            PanelCard("Live Demo") {
                WidgetLiveDemo(widget)
            }
        }
        item {
            PanelCard("Variants & States") {
                WidgetVariantDemo(widget)
            }
        }
        item {
            PanelCard("Implementation Notes") {
                if (widget.notes.isEmpty()) {
                    Text("No extra notes for this widget yet.", style = MaterialTheme.typography.bodyMedium)
                } else {
                    widget.notes.forEach { note ->
                        Text("• $note", style = MaterialTheme.typography.bodyMedium)
                    }
                }
            }
        }
    }
}

@Composable
private fun PanelCard(
    title: String,
    content: @Composable androidx.compose.foundation.layout.ColumnScope.() -> Unit,
) {
    ElevatedCard(shape = RoundedCornerShape(24.dp)) {
        androidx.compose.foundation.layout.Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(20.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            Text(title, style = MaterialTheme.typography.titleLarge)
            content()
        }
    }
}

@Composable
fun FlowTagRow(tags: List<String>) {
    androidx.compose.foundation.layout.FlowRow(
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        tags.forEach { MetaChip(it) }
    }
}

@Composable
fun MetaChip(text: String) {
    Surface(
        shape = RoundedCornerShape(999.dp),
        color = MaterialTheme.colorScheme.surfaceVariant,
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.labelLarge,
            modifier = Modifier.padding(horizontal = 12.dp, vertical = 6.dp),
        )
    }
}

fun sectionIcon(section: AppSection): ImageVector = sectionIcons.getValue(section)
