package com.example.jet_gallery.ui.gallery

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class WidgetDemoRegistryTest {

    @Test
    fun registry_count_matches_expected_widget_scope() {
        assertEquals(EXPECTED_WIDGET_COUNT, WidgetDemoRegistry.demos.size)
    }

    @Test
    fun excluded_web_only_pages_are_not_present() {
        val demoIds = WidgetDemoRegistry.demos.map { it.id }.toSet()
        WidgetDemoRegistry.excludedWebOnlyPages.forEach { excluded ->
            assertFalse("Excluded page should not exist in widget demos: $excluded", demoIds.contains(excluded))
        }
    }

    @Test
    fun every_non_home_section_has_at_least_one_demo() {
        AppSection.entries.filter { it != AppSection.Home }.forEach { section ->
            assertTrue("Section should have demos: $section", WidgetDemoRegistry.firstForSection(section) != null)
        }
    }
}

