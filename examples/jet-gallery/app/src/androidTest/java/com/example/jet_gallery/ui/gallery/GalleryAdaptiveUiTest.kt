package com.example.jet_gallery.ui.gallery

import androidx.compose.ui.test.assertDoesNotExist
import androidx.compose.ui.test.assertExists
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import org.junit.Rule
import org.junit.Test

class GalleryAdaptiveUiTest {

    @get:Rule
    val composeRule = createComposeRule()

    @Test
    fun phone_mode_uses_bottom_navigation() {
        composeRule.setContent {
            JetGalleryApp(windowWidthDpOverride = 400)
        }

        composeRule.onNodeWithTag("bottom-nav").assertExists()
        composeRule.onNodeWithTag("nav-rail").assertDoesNotExist()
    }

    @Test
    fun tablet_mode_uses_navigation_rail() {
        composeRule.setContent {
            JetGalleryApp(windowWidthDpOverride = 1200)
        }

        composeRule.onNodeWithTag("nav-rail").assertExists()
        composeRule.onNodeWithTag("bottom-nav").assertDoesNotExist()
    }

    @Test
    fun native_widget_detail_renders_title() {
        composeRule.setContent {
            WidgetDetailScreen(widget = WidgetDemoRegistry.require("button"))
        }

        composeRule.onNodeWithTag("gallery-detail-title").assertExists()
        composeRule.onNodeWithText("Button").assertExists()
    }

    @Test
    fun composite_widget_detail_renders_title() {
        composeRule.setContent {
            WidgetDetailScreen(widget = WidgetDemoRegistry.require("datatable"))
        }

        composeRule.onNodeWithTag("gallery-detail-title").assertExists()
        composeRule.onNodeWithText("DataTable").assertExists()
    }
}

