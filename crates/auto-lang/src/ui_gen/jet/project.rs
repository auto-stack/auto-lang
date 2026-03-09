//! Android Project Generator
//!
//! Generates complete Android project structure from configuration.
//!
//! ## Output Structure
//!
//! ```text
//! myapp/
//! ├── app/
//! │   ├── src/main/
//! │   │   ├── java/com/example/myapp/
//! │   │   │   ├── MainActivity.kt
//! │   │   │   └── ui/
//! │   │   │       ├── theme/
//! │   │   │       │   ├── Theme.kt
//! │   │   │       │   ├── Color.kt
//! │   │   │       │   └── Type.kt
//! │   │   │       └── widgets/
//! │   │   │           └── *.kt
//! │   │   ├── res/values/strings.xml
//! │   │   └── AndroidManifest.xml
//! │   └── build.gradle.kts
//! ├── build.gradle.kts
//! ├── settings.gradle.kts
//! ├── gradle.properties
//! └── gradle/
//!     └── libs.versions.toml
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use auto_lang::ui_gen::jet::project::{JetProjectConfig, ProjectGenerator, ThemeColors};
//!
//! // Create with defaults
//! let config = JetProjectConfig::new("MyApp");
//!
//! // Or customize everything
//! let config = JetProjectConfig::new("MyApp")
//!     .with_application_id("com.company.myapp")
//!     .with_version("2.0.0")
//!     .with_sdk_versions(26, 34, 34)
//!     .with_theme(ThemeColors::new("#6750A4", "#625B71"))
//!     .with_dependency("coil", "2.5.0")
//!     .with_widget("Counter");
//!
//! // Generate all project files
//! let mut gen = ProjectGenerator::with_config(config);
//! let files = gen.generate();
//!
//! // files is a HashMap<String, String> of path -> content
//! assert!(files.contains_key("app/build.gradle.kts"));
//! ```

use std::collections::HashMap;

/// Android project configuration
///
/// Configuration for generating a complete Android project with Jetpack Compose.
/// All fields have sensible defaults for a modern Material3 Android app.
///
/// # Defaults
///
/// - Package: `com.example.{name.lowercase()}`
/// - Version: `"1.0.0"`
/// - SDK: minSdk 24, compileSdk/targetSdk 34
/// - Kotlin: 1.9.0
/// - Compose BOM: 2024.02.00
/// - Material3: 1.2.0
/// - AGP: 8.2.2
///
/// # Example
///
/// ```rust
/// use auto_lang::ui_gen::jet::project::JetProjectConfig;
///
/// // Simple usage
/// let config = JetProjectConfig::new("MyApp");
/// assert_eq!(config.application_id, "com.example.myapp");
///
/// // Custom configuration
/// let config = JetProjectConfig::new("MyApp")
///     .with_application_id("com.company.myapp")
///     .with_kotlin_version("1.9.22");
/// ```
#[derive(Debug, Clone)]
pub struct JetProjectConfig {
    /// Project name
    pub name: String,

    /// Version string (e.g., "1.0.0")
    pub version: String,

    /// Application ID / package name (e.g., "com.example.myapp")
    pub application_id: String,

    /// Minimum SDK version (default: 24)
    pub min_sdk: u32,

    /// Compile SDK version (default: 34)
    pub compile_sdk: u32,

    /// Target SDK version (default: 34)
    pub target_sdk: u32,

    /// Kotlin version (default: "1.9.0")
    pub kotlin_version: String,

    /// Compose Compiler version (default: "1.5.0")
    pub compose_compiler_version: String,

    /// Compose BOM version (default: "2024.02.00")
    pub compose_bom_version: String,

    /// Material3 version (default: "1.2.0")
    pub material3_version: String,

    /// Activity Compose version (default: "1.8.2")
    pub activity_compose_version: String,

    /// Android Gradle Plugin version (default: "8.2.2")
    pub agp_version: String,

    /// Theme colors (optional)
    pub theme: Option<ThemeColors>,

    /// Additional dependencies (name -> version)
    pub dependencies: HashMap<String, String>,

    /// Widget files to include (filename -> content)
    pub widgets: Vec<String>,
}

impl Default for JetProjectConfig {
    fn default() -> Self {
        Self {
            name: "MyApp".to_string(),
            version: "1.0.0".to_string(),
            application_id: "com.example.myapp".to_string(),
            min_sdk: 24,
            compile_sdk: 34,
            target_sdk: 34,
            kotlin_version: "1.9.0".to_string(),
            compose_compiler_version: "1.5.0".to_string(),
            compose_bom_version: "2024.02.00".to_string(),
            material3_version: "1.2.0".to_string(),
            activity_compose_version: "1.8.2".to_string(),
            agp_version: "8.2.2".to_string(),
            theme: None,
            dependencies: HashMap::new(),
            widgets: Vec::new(),
        }
    }
}

impl JetProjectConfig {
    /// Create a new config with the given project name
    pub fn new(name: &str) -> Self {
        let application_id = format!("com.example.{}", name.to_lowercase().replace('-', "_"));
        Self {
            name: name.to_string(),
            application_id,
            ..Default::default()
        }
    }

    /// Set application ID (builder pattern)
    pub fn with_application_id(mut self, id: &str) -> Self {
        self.application_id = id.to_string();
        self
    }

    /// Set version (builder pattern)
    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    /// Set SDK versions (builder pattern)
    pub fn with_sdk_versions(mut self, min: u32, compile: u32, target: u32) -> Self {
        self.min_sdk = min;
        self.compile_sdk = compile;
        self.target_sdk = target;
        self
    }

    /// Set Kotlin version (builder pattern)
    pub fn with_kotlin_version(mut self, version: &str) -> Self {
        self.kotlin_version = version.to_string();
        self
    }

    /// Set theme colors (builder pattern)
    pub fn with_theme(mut self, theme: ThemeColors) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Add dependency (builder pattern)
    pub fn with_dependency(mut self, name: &str, version: &str) -> Self {
        self.dependencies.insert(name.to_string(), version.to_string());
        self
    }

    /// Add widget (builder pattern)
    pub fn with_widget(mut self, widget_name: &str) -> Self {
        self.widgets.push(widget_name.to_string());
        self
    }

    /// Get package path from application ID (e.g., "com/example/myapp")
    pub fn package_path(&self) -> String {
        self.application_id.replace('.', "/")
    }

    /// Get theme name (based on project name)
    pub fn theme_name(&self) -> String {
        format!("{}Theme", self.name)
    }
}

/// Theme color configuration for Material3
///
/// Defines the primary, secondary, and optional tertiary colors
/// for the generated app theme. Colors are specified as hex strings.
///
/// # Example
///
/// ```rust
/// use auto_lang::ui_gen::jet::project::ThemeColors;
///
/// // Create with primary and secondary
/// let theme = ThemeColors::new("#6750A4", "#625B71");
///
/// // Use Material3 defaults
/// let default_theme = ThemeColors::material3_default();
/// ```
#[derive(Debug, Clone)]
pub struct ThemeColors {
    /// Primary color (hex string, e.g., "#6750A4")
    pub primary: String,

    /// Secondary color (hex string)
    pub secondary: String,

    /// Tertiary color (optional)
    pub tertiary: Option<String>,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            primary: "#6750A4".to_string(),   // Purple40
            secondary: "#625B71".to_string(), // PurpleGrey40
            tertiary: Some("#7D5260".to_string()), // Pink40
        }
    }
}

impl ThemeColors {
    /// Create new theme colors
    pub fn new(primary: &str, secondary: &str) -> Self {
        Self {
            primary: primary.to_string(),
            secondary: secondary.to_string(),
            tertiary: None,
        }
    }

    /// Create Material3 default colors
    pub fn material3_default() -> Self {
        Self::default()
    }
}

/// Android project generator
///
/// Generates a complete Android project structure from configuration.
/// The generated project uses:
/// - Kotlin 1.9.x
/// - Jetpack Compose with Material3
/// - Gradle with Kotlin DSL
/// - Version catalogs for dependency management
///
/// # Example
///
/// ```rust
/// use auto_lang::ui_gen::jet::project::{JetProjectConfig, ProjectGenerator};
///
/// let config = JetProjectConfig::new("MyApp")
///     .with_application_id("com.company.myapp");
///
/// let mut gen = ProjectGenerator::with_config(config);
/// let files = gen.generate();
///
/// // Access generated files
/// assert!(files.contains_key("build.gradle.kts"));
/// assert!(files.contains_key("app/build.gradle.kts"));
/// ```
pub struct ProjectGenerator {
    /// Project configuration
    config: JetProjectConfig,

    /// Generated files (path -> content)
    files: HashMap<String, String>,
}

impl ProjectGenerator {
    /// Create a new ProjectGenerator with default config
    ///
    /// Creates a generator with `JetProjectConfig::default()`:
    /// - Name: "MyApp"
    /// - Package: "com.example.myapp"
    /// - SDK: 24/34/34
    pub fn new() -> Self {
        Self {
            config: JetProjectConfig::default(),
            files: HashMap::new(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: JetProjectConfig) -> Self {
        Self {
            config,
            files: HashMap::new(),
        }
    }

    /// Get current configuration
    pub fn config(&self) -> &JetProjectConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: JetProjectConfig) {
        self.config = config;
    }

    /// Generate all project files
    pub fn generate(&mut self) -> HashMap<String, String> {
        self.files.clear();

        // Generate root level files
        self.generate_root_build_gradle();
        self.generate_settings_gradle();
        self.generate_gradle_properties();
        self.generate_libs_versions_toml();

        // Generate app level files
        self.generate_app_build_gradle();
        self.generate_android_manifest();
        self.generate_main_activity();
        self.generate_strings_xml();

        // Generate theme files
        self.generate_color_kt();
        self.generate_type_kt();
        self.generate_theme_kt();

        self.files.clone()
    }

    /// Get generated files
    pub fn files(&self) -> &HashMap<String, String> {
        &self.files
    }

    /// Add a file to the project
    fn add_file(&mut self, path: &str, content: &str) {
        self.files.insert(path.to_string(), content.to_string());
    }

    // =========================================================================
    // Root Level Files
    // =========================================================================

    /// Generate root build.gradle.kts
    fn generate_root_build_gradle(&mut self) {
        let content = format!(
            r#"// Top-level build file where you can add configuration options common to all sub-projects/modules.
plugins {{
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.kotlin.android) apply false
}}
"#,
        );
        self.add_file("build.gradle.kts", &content);
    }

    /// Generate settings.gradle.kts
    fn generate_settings_gradle(&mut self) {
        let content = r#"pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "MyApp"
include(":app")
"#;
        self.add_file("settings.gradle.kts", content);
    }

    /// Generate gradle.properties
    fn generate_gradle_properties(&mut self) {
        let content = r#"# Project-wide Gradle settings.
org.gradle.jvmargs=-Xmx2048m -Dfile.encoding=UTF-8
android.useAndroidX=true
kotlin.code.style=official
android.nonTransitiveRClass=true
"#;
        self.add_file("gradle.properties", content);
    }

    /// Generate gradle/libs.versions.toml
    fn generate_libs_versions_toml(&mut self) {
        let content = format!(
            r#"[versions]
agp = "{}"
kotlin = "{}"
compose-bom = "{}"
material3 = "{}"
activity-compose = "{}"

[libraries]
compose-bom = {{ group = "androidx.compose", name = "compose-bom", version.ref = "compose-bom" }}
compose-ui = {{ group = "androidx.compose.ui", name = "ui" }}
compose-ui-graphics = {{ group = "androidx.compose.ui", name = "ui-graphics" }}
compose-ui-tooling = {{ group = "androidx.compose.ui", name = "ui-tooling" }}
compose-ui-tooling-preview = {{ group = "androidx.compose.ui", name = "ui-tooling-preview" }}
compose-ui-test-manifest = {{ group = "androidx.compose.ui", name = "ui-test-manifest" }}
compose-material3 = {{ group = "androidx.compose.material3", name = "material3", version.ref = "material3" }}
activity-compose = {{ group = "androidx.activity", name = "activity-compose", version.ref = "activity-compose" }}
core-ktx = {{ group = "androidx.core", name = "core-ktx", version = "1.12.0" }}
lifecycle-runtime = {{ group = "androidx.lifecycle", name = "lifecycle-runtime-ktx", version = "2.7.0" }}

[plugins]
android-application = {{ id = "com.android.application", version.ref = "agp" }}
kotlin-android = {{ id = "org.jetbrains.kotlin.android", version.ref = "kotlin" }}
"#,
            self.config.agp_version,
            self.config.kotlin_version,
            self.config.compose_bom_version,
            self.config.material3_version,
            self.config.activity_compose_version,
        );
        self.add_file("gradle/libs.versions.toml", &content);
    }

    // =========================================================================
    // App Level Files
    // =========================================================================

    /// Generate app/build.gradle.kts
    fn generate_app_build_gradle(&mut self) {
        let namespace = &self.config.application_id;
        let application_id = &self.config.application_id;
        let min_sdk = self.config.min_sdk;
        let compile_sdk = self.config.compile_sdk;
        let target_sdk = self.config.target_sdk;
        let version_name = &self.config.version;
        let compose_compiler = &self.config.compose_compiler_version;

        // Generate additional dependencies
        let mut extra_deps = String::new();
        for (name, _version) in &self.config.dependencies {
            match name.as_str() {
                "coil" => {
                    extra_deps.push_str("    implementation(\"io.coil-kt:coil-compose:2.5.0\")\n");
                }
                "viewmodel" => {
                    extra_deps.push_str("    implementation(\"androidx.lifecycle:lifecycle-viewmodel-compose:2.7.0\")\n");
                }
                "navigation" => {
                    extra_deps.push_str("    implementation(\"androidx.navigation:navigation-compose:2.7.6\")\n");
                }
                _ => {}
            }
        }

        let content = format!(
            r#"plugins {{
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
}}

android {{
    namespace = "{}"
    compileSdk = {}

    defaultConfig {{
        applicationId = "{}"
        minSdk = {}
        targetSdk = {}
        versionCode = 1
        versionName = "{}"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        vectorDrawables {{
            useSupportLibrary = true
        }}
    }}

    buildTypes {{
        release {{
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }}
    }}

    compileOptions {{
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }}

    kotlinOptions {{
        jvmTarget = "17"
    }}

    buildFeatures {{
        compose = true
    }}

    composeOptions {{
        kotlinCompilerExtensionVersion = "{}"
    }}

    packaging {{
        resources {{
            excludes += "/META-INF/{{AL2.0,LGPL2.1}}"
        }}
    }}
}}

dependencies {{
    implementation(libs.core.ktx)
    implementation(libs.lifecycle.runtime)
    implementation(libs.activity.compose)
    implementation(platform(libs.compose.bom))
    implementation(libs.compose.ui)
    implementation(libs.compose.ui.graphics)
    implementation(libs.compose.ui.tooling.preview)
    implementation(libs.compose.material3)
{}
    debugImplementation(libs.compose.ui.tooling)
    debugImplementation(libs.compose.ui.test.manifest)
}}
"#,
            namespace,
            compile_sdk,
            application_id,
            min_sdk,
            target_sdk,
            version_name,
            compose_compiler,
            extra_deps,
        );
        self.add_file("app/build.gradle.kts", &content);
    }

    /// Generate AndroidManifest.xml
    fn generate_android_manifest(&mut self) {
        let app_name = &self.config.name;

        let content = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">

    <application
        android:allowBackup="true"
        android:icon="@mipmap/ic_launcher"
        android:label="@string/app_name"
        android:roundIcon="@mipmap/ic_launcher_round"
        android:supportsRtl="true"
        android:theme="@style/Theme.{app_name}">
        <activity
            android:name=".MainActivity"
            android:exported="true"
            android:theme="@style/Theme.{app_name}">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
    </application>

</manifest>
"#,
        );
        self.add_file("app/src/main/AndroidManifest.xml", &content);
    }

    /// Generate MainActivity.kt
    fn generate_main_activity(&mut self) {
        let package = &self.config.application_id;
        let theme_name = self.config.theme_name();
        let app_name = &self.config.name;

        // Generate widget imports
        let widget_imports: Vec<String> = self
            .config
            .widgets
            .iter()
            .map(|w| format!("import {package}.ui.widgets.{w}"))
            .collect();
        let widget_imports_str = widget_imports.join("\n");

        // Generate widget calls in content
        let widget_calls: Vec<String> = self
            .config
            .widgets
            .iter()
            .map(|w| format!("                {w}()"))
            .collect();
        let widget_calls_str = if widget_calls.is_empty() {
            "                // Add your widgets here".to_string()
        } else {
            widget_calls.join("\n")
        };

        let content = format!(
            r#"package {package}

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import {package}.ui.theme.{theme_name}
{widget_imports_str}

class MainActivity : ComponentActivity() {{
    override fun onCreate(savedInstanceState: Bundle?) {{
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {{
            {theme_name} {{
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {{
{widget_calls_str}
                }}
            }}
        }}
    }}
}}
"#,
        );
        self.add_file(
            &format!("app/src/main/java/{}/MainActivity.kt", self.config.package_path()),
            &content,
        );
    }

    /// Generate strings.xml
    fn generate_strings_xml(&mut self) {
        let app_name = &self.config.name;

        let content = format!(
            r#"<resources>
    <string name="app_name">{app_name}</string>
</resources>
"#,
        );
        self.add_file("app/src/main/res/values/strings.xml", &content);
    }

    // =========================================================================
    // Theme Files
    // =========================================================================

    /// Generate Color.kt
    fn generate_color_kt(&mut self) {
        let package = &self.config.application_id;

        // Use custom colors if provided, otherwise use Material3 defaults
        let (primary, secondary, tertiary) = if let Some(ref theme) = self.config.theme {
            (
                theme.primary.as_str(),
                theme.secondary.as_str(),
                theme.tertiary.as_deref().unwrap_or("#7D5260"),
            )
        } else {
            ("#6750A4", "#625B71", "#7D5260")
        };

        // Convert hex to Compose Color format
        let primary_40 = hex_to_compose_color(primary);
        let secondary_40 = hex_to_compose_color(secondary);
        let tertiary_40 = hex_to_compose_color(tertiary);

        // Generate light variants (80% lighter)
        let primary_80 = lighten_hex(primary);
        let secondary_80 = lighten_hex(secondary);
        let tertiary_80 = lighten_hex(tertiary);

        let content = format!(
            r#"package {package}.ui.theme

import androidx.compose.ui.graphics.Color

val Purple80 = Color(0x{primary_80})
val PurpleGrey80 = Color(0x{secondary_80})
val Pink80 = Color(0x{tertiary_80})

val Purple40 = Color(0x{primary_40})
val PurpleGrey40 = Color(0x{secondary_40})
val Pink40 = Color(0x{tertiary_40})
"#,
        );
        self.add_file(
            &format!("app/src/main/java/{}/ui/theme/Color.kt", self.config.package_path()),
            &content,
        );
    }

    /// Generate Type.kt
    fn generate_type_kt(&mut self) {
        let package = &self.config.application_id;

        let content = format!(
            r#"package {package}.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

// Set of Material typography styles to start with
val Typography = Typography(
    bodyLarge = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Normal,
        fontSize = 16.sp,
        lineHeight = 24.sp,
        letterSpacing = 0.5.sp
    )
    /* Other default text styles to override
    titleLarge = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Normal,
        fontSize = 22.sp,
        lineHeight = 28.sp,
        letterSpacing = 0.sp
    ),
    labelSmall = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Medium,
        fontSize = 11.sp,
        lineHeight = 16.sp,
        letterSpacing = 0.5.sp
    )
    */
)
"#,
        );
        self.add_file(
            &format!("app/src/main/java/{}/ui/theme/Type.kt", self.config.package_path()),
            &content,
        );
    }

    /// Generate Theme.kt
    fn generate_theme_kt(&mut self) {
        let package = &self.config.application_id;
        let theme_name = self.config.theme_name();

        let content = format!(
            r#"package {package}.ui.theme

import android.app.Activity
import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat

private val DarkColorScheme = darkColorScheme(
    primary = Purple80,
    secondary = PurpleGrey80,
    tertiary = Pink80
)

private val LightColorScheme = lightColorScheme(
    primary = Purple40,
    secondary = PurpleGrey40,
    tertiary = Pink40
)

@Composable
fun {theme_name}(
    darkTheme: Boolean = isSystemInDarkTheme(),
    // Dynamic color is available on Android 12+
    dynamicColor: Boolean = true,
    content: @Composable () -> Unit
) {{
    val colorScheme = when {{
        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {{
            val context = LocalContext.current
            if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        }}
        darkTheme -> DarkColorScheme
        else -> LightColorScheme
    }}
    val view = LocalView.current
    if (!view.isInEditMode) {{
        SideEffect {{
            val window = (view.context as Activity).window
            window.statusBarColor = colorScheme.primary.toArgb()
            WindowCompat.getInsetsController(window, view).isAppearanceLightStatusBars = !darkTheme
        }}
    }}

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content
    )
}}
"#,
        );
        self.add_file(
            &format!("app/src/main/java/{}/ui/theme/Theme.kt", self.config.package_path()),
            &content,
        );
    }
}

impl Default for ProjectGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// =========================================================================
// Helper Functions
// =========================================================================

/// Convert hex color string to Compose Color hex format
/// Input: "#6750A4" -> Output: "FF6750A4"
fn hex_to_compose_color(hex: &str) -> String {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        format!("FF{}", hex.to_uppercase())
    } else if hex.len() == 8 {
        hex.to_uppercase()
    } else {
        "FF6750A4".to_string() // Default purple
    }
}

/// Lighten a hex color for dark theme variants
/// This is a simplified lightening that adds 0x40 to each RGB component
fn lighten_hex(hex: &str) -> String {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return "FFD0BCFF".to_string(); // Default light purple
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0x67);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0x50);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0xA4);

    // Lighten by blending with white (simplified)
    let r_light = ((r as u16 + 0x40).min(0xD0)) as u8;
    let g_light = ((g as u16 + 0x6C).min(0xBC)) as u8;
    let b_light = ((b as u16 + 0x5B).min(0xFF)) as u8;

    format!("FF{:02X}{:02X}{:02X}", r_light, g_light, b_light)
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jet_project_config_default() {
        let config = JetProjectConfig::default();
        assert_eq!(config.name, "MyApp");
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.application_id, "com.example.myapp");
        assert_eq!(config.min_sdk, 24);
        assert_eq!(config.compile_sdk, 34);
        assert_eq!(config.target_sdk, 34);
    }

    #[test]
    fn test_jet_project_config_builder() {
        let config = JetProjectConfig::new("TestApp")
            .with_version("2.0.0")
            .with_application_id("com.test.app")
            .with_sdk_versions(26, 34, 34);

        assert_eq!(config.name, "TestApp");
        assert_eq!(config.version, "2.0.0");
        assert_eq!(config.application_id, "com.test.app");
        assert_eq!(config.min_sdk, 26);
    }

    #[test]
    fn test_jet_project_config_package_path() {
        let config = JetProjectConfig::new("MyApp");
        assert_eq!(config.package_path(), "com/example/myapp");

        let config2 = JetProjectConfig::new("TestApp").with_application_id("org.example.test");
        assert_eq!(config2.package_path(), "org/example/test");
    }

    #[test]
    fn test_jet_project_config_theme_name() {
        let config = JetProjectConfig::new("CoolApp");
        assert_eq!(config.theme_name(), "CoolAppTheme");
    }

    #[test]
    fn test_theme_colors_default() {
        let theme = ThemeColors::default();
        assert_eq!(theme.primary, "#6750A4");
        assert_eq!(theme.secondary, "#625B71");
        assert_eq!(theme.tertiary, Some("#7D5260".to_string()));
    }

    #[test]
    fn test_theme_colors_new() {
        let theme = ThemeColors::new("#FF0000", "#00FF00");
        assert_eq!(theme.primary, "#FF0000");
        assert_eq!(theme.secondary, "#00FF00");
        assert_eq!(theme.tertiary, None);
    }

    #[test]
    fn test_project_generator_new() {
        let gen = ProjectGenerator::new();
        assert_eq!(gen.config().name, "MyApp");
    }

    #[test]
    fn test_project_generator_generate() {
        let config = JetProjectConfig::new("TestApp")
            .with_application_id("com.test.app")
            .with_widget("Counter")
            .with_widget("TodoList");

        let mut gen = ProjectGenerator::with_config(config);
        let files = gen.generate();

        // Verify essential files are generated
        assert!(files.contains_key("build.gradle.kts"));
        assert!(files.contains_key("settings.gradle.kts"));
        assert!(files.contains_key("gradle.properties"));
        assert!(files.contains_key("gradle/libs.versions.toml"));
        assert!(files.contains_key("app/build.gradle.kts"));
        assert!(files.contains_key("app/src/main/AndroidManifest.xml"));
        assert!(files.contains_key("app/src/main/res/values/strings.xml"));

        // Verify package path in file paths
        assert!(files.contains_key("app/src/main/java/com/test/app/MainActivity.kt"));
        assert!(files.contains_key("app/src/main/java/com/test/app/ui/theme/Color.kt"));
        assert!(files.contains_key("app/src/main/java/com/test/app/ui/theme/Type.kt"));
        assert!(files.contains_key("app/src/main/java/com/test/app/ui/theme/Theme.kt"));
    }

    #[test]
    fn test_generate_root_build_gradle() {
        let mut gen = ProjectGenerator::new();
        gen.generate_root_build_gradle();
        let files = gen.files();

        let content = files.get("build.gradle.kts").unwrap();
        assert!(content.contains("plugins"));
        assert!(content.contains("android.application"));
        assert!(content.contains("kotlin.android"));
    }

    #[test]
    fn test_generate_libs_versions_toml() {
        let mut gen = ProjectGenerator::new();
        gen.generate_libs_versions_toml();
        let files = gen.files();

        let content = files.get("gradle/libs.versions.toml").unwrap();
        assert!(content.contains("[versions]"));
        assert!(content.contains("[libraries]"));
        assert!(content.contains("[plugins]"));
        assert!(content.contains("compose-bom"));
        assert!(content.contains("material3"));
    }

    #[test]
    fn test_generate_app_build_gradle() {
        let config = JetProjectConfig::new("TestApp").with_application_id("com.test.app");
        let mut gen = ProjectGenerator::with_config(config);
        gen.generate_app_build_gradle();
        let files = gen.files();

        let content = files.get("app/build.gradle.kts").unwrap();
        assert!(content.contains("namespace = \"com.test.app\""));
        assert!(content.contains("applicationId = \"com.test.app\""));
        assert!(content.contains("minSdk = 24"));
        assert!(content.contains("compose = true"));
    }

    #[test]
    fn test_generate_android_manifest() {
        let config = JetProjectConfig::new("MyApp");
        let mut gen = ProjectGenerator::with_config(config);
        gen.generate_android_manifest();
        let files = gen.files();

        let content = files.get("app/src/main/AndroidManifest.xml").unwrap();
        assert!(content.contains("<?xml version"));
        assert!(content.contains(".MainActivity"));
        assert!(content.contains("android.intent.action.MAIN"));
    }

    #[test]
    fn test_generate_main_activity() {
        let config = JetProjectConfig::new("TestApp")
            .with_application_id("com.test.app")
            .with_widget("Counter");

        let mut gen = ProjectGenerator::with_config(config);
        gen.generate_main_activity();
        let files = gen.files();

        let content = files.get("app/src/main/java/com/test/app/MainActivity.kt").unwrap();
        assert!(content.contains("package com.test.app"));
        assert!(content.contains("class MainActivity"));
        assert!(content.contains("TestAppTheme"));
        assert!(content.contains("import com.test.app.ui.widgets.Counter"));
    }

    #[test]
    fn test_generate_theme_kt() {
        let config = JetProjectConfig::new("TestApp").with_application_id("com.test.app");
        let mut gen = ProjectGenerator::with_config(config);
        gen.generate_theme_kt();
        let files = gen.files();

        let content = files.get("app/src/main/java/com/test/app/ui/theme/Theme.kt").unwrap();
        assert!(content.contains("package com.test.app.ui.theme"));
        assert!(content.contains("fun TestAppTheme"));
        assert!(content.contains("MaterialTheme"));
    }

    #[test]
    fn test_hex_to_compose_color() {
        assert_eq!(hex_to_compose_color("#6750A4"), "FF6750A4");
        assert_eq!(hex_to_compose_color("6750A4"), "FF6750A4");
        assert_eq!(hex_to_compose_color("#FF6750A4"), "FF6750A4");
    }

    #[test]
    fn test_lighten_hex() {
        let result = lighten_hex("#6750A4");
        // Should produce a lighter version
        assert!(result.starts_with("FF"));
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn test_project_with_dependencies() {
        let config = JetProjectConfig::new("TestApp")
            .with_dependency("coil", "2.5.0")
            .with_dependency("viewmodel", "2.7.0");

        let mut gen = ProjectGenerator::with_config(config);
        gen.generate_app_build_gradle();
        let files = gen.files();

        let content = files.get("app/build.gradle.kts").unwrap();
        assert!(content.contains("coil-compose"));
        assert!(content.contains("lifecycle-viewmodel-compose"));
    }

    #[test]
    fn test_project_with_custom_theme() {
        let theme = ThemeColors::new("#FF0000", "#00FF00");
        let config = JetProjectConfig::new("TestApp").with_theme(theme);

        let mut gen = ProjectGenerator::with_config(config);
        gen.generate_color_kt();
        let files = gen.files();

        let content = files.get("app/src/main/java/com/example/testapp/ui/theme/Color.kt").unwrap();
        assert!(content.contains("FF0000")); // Custom primary
    }
}
