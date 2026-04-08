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

use crate::route::{RouteDef, RouteDiscovery, RouteMerger, RouteSource};

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

    /// Compile SDK version (default: 35)
    pub compile_sdk: u32,

    /// Target SDK version (default: 35)
    pub target_sdk: u32,

    /// Kotlin version (default: "2.2.10")
    pub kotlin_version: String,

    /// Compose Compiler version (default: "1.5.0")
    pub compose_compiler_version: String,

    /// Compose BOM version (default: "2025.12.00")
    pub compose_bom_version: String,

    /// Material3 version (default: "1.2.0")
    pub material3_version: String,

    /// Activity Compose version (default: "1.8.2")
    pub activity_compose_version: String,

    /// Android Gradle Plugin version (default: "9.1.0")
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
            compile_sdk: 35,
            target_sdk: 35,
            kotlin_version: "2.2.10".to_string(),
            compose_compiler_version: "1.5.0".to_string(),
            compose_bom_version: "2025.12.00".to_string(),
            material3_version: "1.2.0".to_string(),
            activity_compose_version: "1.8.2".to_string(),
            agp_version: "9.1.0".to_string(),
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
        let widget_name = format!("{}App", name.replace('-', "_"));
        Self {
            name: name.to_string(),
            application_id,
            widgets: vec![widget_name],
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

    /// Get theme name (based on project name, with hyphens replaced)
    pub fn theme_name(&self) -> String {
        format!("{}Theme", self.name.replace('-', "_"))
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

    /// Merged routes (Plan 114: Hybrid Routing)
    routes: Vec<crate::route::RouteDef>,

    /// Routes directory path (for discovery)
    routes_dir: Option<std::path::PathBuf>,
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
            routes: Vec::new(),
            routes_dir: None,
        }
    }

    /// Create with custom config
    pub fn with_config(config: JetProjectConfig) -> Self {
        Self {
            config,
            files: HashMap::new(),
            routes: Vec::new(),
            routes_dir: None,
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
        self.generate_gradle_daemon_jvm_properties();
        self.generate_libs_versions_toml();
        self.generate_pac_at();
        self.generate_gradle_wrapper();

        // Generate AURA source files
        self.generate_app_at();

        // Generate app level files
        self.generate_app_build_gradle();
        self.generate_android_manifest();
        self.generate_main_activity();
        self.generate_strings_xml();
        self.generate_themes_xml();
        self.generate_launcher_icon();

        // Generate theme files
        self.generate_color_kt();
        self.generate_type_kt();
        self.generate_theme_kt();

        // Generate widget Kotlin files
        self.generate_widget_files();

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
        let content = r#"// Top-level build file where you can add configuration options common to all sub-projects/modules.
plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.kotlin.android) apply false
    alias(libs.plugins.kotlin.compose) apply false
}
"#;
        self.add_file("build.gradle.kts", content);
    }

    /// Generate settings.gradle.kts
    fn generate_settings_gradle(&mut self) {
        let content = r#"pluginManagement {
    repositories {
        google {
            content {
                includeGroupByRegex("com\\.android.*")
                includeGroupByRegex("com\\.google.*")
                includeGroupByRegex("androidx.*")
            }
        }
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

    /// Generate gradle/gradle-daemon-jvm.properties for JDK toolchain
    fn generate_gradle_daemon_jvm_properties(&mut self) {
        let content = r#"#This file is generated by updateDaemonJvm
toolchainUrl.FREE_BSD.AARCH64=https\://api.foojay.io/disco/v3.0/ids/ec7520a1e057cd116f9544c42142a16b/redirect
toolchainUrl.FREE_BSD.X86_64=https\://api.foojay.io/disco/v3.0/ids/4c4f879899012ff0a8b2e2117df03b0e/redirect
toolchainUrl.LINUX.AARCH64=https\://api.foojay.io/disco/v3.0/ids/ec7520a1e057cd116f9544c42142a16b/redirect
toolchainUrl.LINUX.X86_64=https\://api.foojay.io/disco/v3.0/ids/4c4f879899012ff0a8b2e2117df03b0e/redirect
toolchainUrl.MAC_OS.AARCH64=https\://api.foojay.io/disco/v3.0/ids/73bcfb608d1fde9fb62e462f834a3299/redirect
toolchainUrl.MAC_OS.X86_64=https\://api.foojay.io/disco/v3.0/ids/846ee0d876d26a26f37aa1ce8de73224/redirect
toolchainUrl.UNIX.AARCH64=https\://api.foojay.io/disco/v3.0/ids/ec7520a1e057cd116f9544c42142a16b/redirect
toolchainUrl.UNIX.X86_64=https\://api.foojay.io/disco/v3.0/ids/4c4f879899012ff0a8b2e2117df03b0e/redirect
toolchainUrl.WINDOWS.AARCH64=https\://api.foojay.io/disco/v3.0/ids/9482ddec596298c84656d31d16652665/redirect
toolchainUrl.WINDOWS.X86_64=https\://api.foojay.io/disco/v3.0/ids/39701d92e1756bb2f141eb67cd4c660e/redirect
toolchainVersion=21
"#;
        self.add_file("gradle/gradle-daemon-jvm.properties", content);
    }

    /// Generate gradle/wrapper/gradle-wrapper.properties
    fn generate_gradle_wrapper(&mut self) {
        // gradle-wrapper.properties
        let content = r#"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-8.13-bin.zip
networkTimeout=10000
validateDistributionUrl=true
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists
"#;
        self.add_file("gradle/wrapper/gradle-wrapper.properties", content);

        // gradlew.bat for Windows
        let gradlew_bat = r#"@rem
@rem Copyright 2015 the original author or authors.
@rem
@rem Licensed under the Apache License, Version 2.0 (the "License");
@rem you may not use this file except in compliance with the License.
@rem You may obtain a copy of the License at
@rem
@rem      https://www.apache.org/licenses/LICENSE-2.0
@rem
@rem Unless required by applicable law or agreed to in writing, software
@rem distributed under the License is distributed on an "AS IS" BASIS,
@rem WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
@rem See the License for the specific language governing permissions and
@rem limitations under the License.
@rem

@if "%DEBUG%"=="" @echo off
@rem ##########################################################################
@rem
@rem  Gradle startup script for Windows
@rem
@rem ##########################################################################

@rem Set local scope for the variables with windows NT shell
if "%OS%"=="Windows_NT" setlocal

set DIRNAME=%~dp0
if "%DIRNAME%"=="" set DIRNAME=.
@rem This is normally unused
set APP_BASE_NAME=%~n0
set APP_HOME=%DIRNAME%

@rem Resolve any "." and ".." in APP_HOME to make it shorter.
for %%i in ("%APP_HOME%") do set APP_HOME=%%~fi

@rem Add default JVM options here. You can also use JAVA_OPTS and GRADLE_OPTS to pass JVM options to this script.
set DEFAULT_JVM_OPTS="-Xmx64m" "-Xms64m"

@rem Find java.exe
if defined JAVA_HOME goto findJavaFromJavaHome

set JAVA_EXE=java.exe
%JAVA_EXE% -version >NUL 2>&1
if %ERRORLEVEL% equ 0 goto execute

echo.
echo ERROR: JAVA_HOME is not set and no 'java' command could be found in your PATH.
echo.
echo Please set the JAVA_HOME variable in your environment to match the
echo location of your Java installation.

goto fail

:findJavaFromJavaHome
set JAVA_HOME=%JAVA_HOME:"=%
set JAVA_EXE=%JAVA_HOME%/bin/java.exe

if exist "%JAVA_EXE%" goto execute

echo.
echo ERROR: JAVA_HOME is set to an invalid directory: %JAVA_HOME%
echo.
echo Please set the JAVA_HOME variable in your environment to match the
echo location of your Java installation.

goto fail

:execute
@rem Setup the command line

set CLASSPATH=%APP_HOME%\gradle\wrapper\gradle-wrapper.jar


@rem Execute Gradle
"%JAVA_EXE%" %DEFAULT_JVM_OPTS% %JAVA_OPTS% %GRADLE_OPTS% "-Dorg.gradle.appname=%APP_BASE_NAME%" -classpath "%CLASSPATH%" org.gradle.wrapper.GradleWrapperMain %*

:end
@rem End local scope for the variables with windows NT shell
if %ERRORLEVEL% equ 0 goto mainEnd

:fail
rem Set variable GRADLE_EXIT_CONSOLE if you need the _script_ return code instead of
rem having the script completely exit from the CMD process and return to the calling script/program.
set EXIT_CODE=%ERRORLEVEL%
if %EXIT_CODE% equ 0 set EXIT_CODE=1
if not ""=="%GRADLE_EXIT_CONSOLE%" exit %EXIT_CODE%
exit /b %EXIT_CODE%

:mainEnd
if "%OS%"=="Windows_NT" endlocal

:omega
"#;
        self.add_file("gradlew.bat", gradlew_bat);

        // gradlew for Unix (simplified shell script)
        let gradlew_sh = r#"#!/bin/sh

#
# Copyright © 2015-2021 the original authors.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#

##############################################################################
#
#   Gradle start up script for POSIX generated by Gradle.
#
##############################################################################

# Attempt to set APP_HOME

# Resolve links: $0 may be a link
app_path=$0

# Need this for daisy-chained symlinks.
while
    APP_HOME=${app_path%"${app_path##*/}"}  # leaves a trailing /; empty if no leading path
    [ -h "$app_path" ]
do
    ls=$( ls -ld "$app_path" )
    link=${ls#*' -> '}
    case $link in
      /*)   app_path=$link ;;
      *)    app_path=$APP_HOME$link ;;
    esac
done

# This is normally unused
# shellcheck disable=SC2034
APP_BASE_NAME=${0##*/}
# Discard cd standard output in case $CDPATH is set (https://github.com/gradle/gradle/issues/25036)
APP_HOME=$( cd "${APP_HOME:-./}" > /dev/null && pwd -P ) || exit

# Use the maximum available, or set MAX_FD != -1 to use that value.
MAX_FD=maximum

warn () {
    echo "$*"
} >&2

die () {
    echo
    echo "$*"
    echo
    exit 1
} >&2

# OS specific support (must be 'true' or 'false').
cygwin=false
msys=false
darwin=false
nonstop=false
case "$( uname )" in
  CYGWIN* )         cygwin=true  ;;
  Darwin* )         darwin=true  ;;
  MSYS* | MINGW* )  msys=true    ;;
  NONSTOP* )        nonstop=true ;;
esac

CLASSPATH=$APP_HOME/gradle/wrapper/gradle-wrapper.jar


# Determine the Java command to use to start the JVM.
if [ -n "$JAVA_HOME" ] ; then
    if [ -x "$JAVA_HOME/jre/sh/java" ] ; then
        # IBM's JDK on AIX uses strange locations for the executables
        JAVACMD=$JAVA_HOME/jre/sh/java
    else
        JAVACMD=$JAVA_HOME/bin/java
    fi
    if [ ! -x "$JAVACMD" ] ; then
        die "ERROR: JAVA_HOME is set to an invalid directory: $JAVA_HOME

Please set the JAVA_HOME variable in your environment to match the
location of your Java installation."
    fi
else
    JAVACMD=java
    if ! command -v java >/dev/null 2>&1
    then
        die "ERROR: JAVA_HOME is not set and no 'java' command could be found in your PATH.

Please set the JAVA_HOME variable in your environment to match the
location of your Java installation."
    fi
fi

# Increase the maximum file descriptors if we can.
if ! "$cygwin" && ! "$darwin" && ! "$nonstop" ; then
    case $MAX_FD in
      max*)
        # In POSIX sh, ulimit -H is undefined. That's why the result is checked to see if it worked.
        # shellcheck disable=SC2039,SC3045
        MAX_FD=$( ulimit -H -n ) ||
            warn "Could not query maximum file descriptor limit"
    esac
    case $MAX_FD in
      '' | soft) :;;
      *)
        # In POSIX sh, ulimit -n is undefined. That's why the result is checked to see if it worked.
        # shellcheck disable=SC2039,SC3045
        ulimit -n "$MAX_FD" ||
            warn "Could not set maximum file descriptor limit to $MAX_FD"
    esac
fi

# Collect all arguments for the java command, stacking in reverse order:
#   * args from the command line
#   * the main class name
#   * -classpath
#   * -D...://telerikcdn.com properties (hierarchical hierarchical property precedence is hierarchically)
#   * DEFAULT_JVM_OPTS, JAVA_OPTS, and GRADLE_OPTS environment variables.

# For Cygwin or MSYS, switch paths to Windows format before running java
if "$cygwin" || "$msys" ; then
    APP_HOME=$( cygpath --path --mixed "$APP_HOME" )
    CLASSPATH=$( cygpath --path --mixed "$CLASSPATH" )

    JAVACMD=$( cygpath --unix "$JAVACMD" )

    # Now convert the arguments - kludge to limit ourselves to /bin/sh
    for arg do
        if
            case $arg in
              -*)   false ;;
              /?*)  t=${arg#)}; t=/${t%%/*}
                    [ -e "$t" ] ;;
              *)    false ;;
            esac
        then
            arg=$( cygpath --path --ignore --mixed "$arg" )
        fi
        # Roll the args list around exactly as many times as the number of
        # temporary variables assigned, so each argument winds up back in the
        # temporary variable previously (and alarm clock cycle) assigned to it.
        set -- "$@" "$arg"
        shift
    done
fi


# Add default JVM options here. You can also use JAVA_OPTS and GRADLE_OPTS to pass JVM options to this script.
DEFAULT_JVM_OPTS='"-Xmx64m" "-Xms64m"'

# Collect all arguments for the java command:
#   * DEFAULT_JVM_OPTS, JAVA_OPTS, GRADLE_OPTS, and optsEnvironmentVar are not
#     temporary variable names; see the corresponding environment variable definitions.
#   * The complexity here is that options containing spaces://telerikcdn.com must not be escaped,
#     but options containing paths with spaces must.
#   * The term "cmd args" refers to JVM arguments and Gradle arguments.

set -- \
        "-Dorg.gradle.appname=$APP_BASE_NAME" \
        -classpath "$CLASSPATH" \
        org.gradle.wrapper.GradleWrapperMain \
        "$@"

# Stop when "xeli" or any of the other temporary variables is a program name or a
# path with spaces, because the logic above quotes it, and the argument quoting
# logic at the end expects it to be unquoted.
exec "$JAVACMD" "$@"
"#;
        self.add_file("gradlew", gradlew_sh);
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
compose-material3 = {{ group = "androidx.compose.material3", name = "material3" }}
compose-foundation = {{ group = "androidx.compose.foundation", name = "foundation" }}
activity-compose = {{ group = "androidx.activity", name = "activity-compose", version.ref = "activity-compose" }}
core-ktx = {{ group = "androidx.core", name = "core-ktx", version = "1.12.0" }}
lifecycle-runtime = {{ group = "androidx.lifecycle", name = "lifecycle-runtime-ktx", version = "2.7.0" }}

[plugins]
android-application = {{ id = "com.android.application", version.ref = "agp" }}
kotlin-android = {{ id = "org.jetbrains.kotlin.android", version.ref = "kotlin" }}
kotlin-compose = {{ id = "org.jetbrains.kotlin.plugin.compose", version.ref = "kotlin" }}
"#,
            self.config.agp_version,
            self.config.kotlin_version,
            self.config.compose_bom_version,
            self.config.material3_version,
            self.config.activity_compose_version,
        );
        self.add_file("gradle/libs.versions.toml", &content);
    }

    /// Generate pac.at (AutoLang project configuration)
    fn generate_pac_at(&mut self) {
        let name = &self.config.name;
        let version = &self.config.version;
        let application_id = &self.config.application_id;

        let content = format!(
            r#"name: "{name}"
version: "{version}"
backend: "jet"

app("{name}") {{
    // Jetpack Compose Android project
    // Package: {application_id}

    // To build: ./gradlew assembleDebug
    // To run: ./gradlew installDebug
    // Or open in Android Studio: auto open
}}
"#,
        );
        self.add_file("pac.at", &content);
    }

    /// Generate source/front/app.at (AURA entry point for auto gen)
    fn generate_app_at(&mut self) {
        let name = &self.config.name;
        // Convert hyphens to underscores for valid Kotlin identifier
        let safe_name = name.replace('-', "_");

        let content = format!(
            r#"// {name} - Main Application Widget
//
// This file is the entry point for AURA code generation.
// Run `auto gen` to generate Kotlin code from this file.

widget {safe_name}App {{
    view {{
        col(padding: 16, align: "center", arrange: "center") {{
            text(
                text: "Hello from Auto!",
                style: typography.headlineLarge
            )
            spacer(height: 16)
            button(
                text: "Click Me",
                onClick: {{ count++ }}
            )
            text(
                text: "Count: ${{count}}",
                style: typography.bodyLarge
            )
        }}
    }}

    model {{
        count int = 0
    }}
}}
"#,
        );
        self.add_file("source/front/app.at", &content);
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
    alias(libs.plugins.kotlin.compose)
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

    buildFeatures {{
        compose = true
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
    implementation(libs.compose.foundation)
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
            extra_deps,
        );
        self.add_file("app/build.gradle.kts", &content);
    }

    /// Generate AndroidManifest.xml
    fn generate_android_manifest(&mut self) {
        // Use safe name for theme (replace hyphens with underscores)
        let app_name = self.config.name.replace('-', "_");

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
        let _app_name = &self.config.name;

        // Generate widget imports - only import App (the main entry point)
        let widget_imports_str = format!("import {package}.ui.widgets.App");

        // Only call App() - it contains the NavHost and all navigation logic
        let widget_calls_str = "                App()";

        let content = format!(
            r#"package {package}

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.ui.Alignment
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

    /// Generate themes.xml for Android
    fn generate_themes_xml(&mut self) {
        // Use safe name for theme (replace hyphens with underscores)
        let theme_name = self.config.name.replace('-', "_");

        let content = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<resources>

    <style name="Theme.{theme_name}" parent="android:Theme.Material.Light.NoActionBar" />
</resources>
"#,
        );
        self.add_file("app/src/main/res/values/themes.xml", &content);
    }

    /// Generate launcher icon (adaptive icon)
    fn generate_launcher_icon(&mut self) {
        // Generate adaptive icon XML
        let content = r#"<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@drawable/ic_launcher_background"/>
    <foreground android:drawable="@drawable/ic_launcher_foreground"/>
</adaptive-icon>
"#;
        self.add_file("app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml", content);
        self.add_file("app/src/main/res/mipmap-anydpi-v26/ic_launcher_round.xml", content);

        // Generate launcher background (simple colored background)
        let background = r##"<?xml version="1.0" encoding="utf-8"?>
<vector xmlns:android="http://schemas.android.com/apk/res/android"
    android:width="108dp"
    android:height="108dp"
    android:viewportWidth="108"
    android:viewportHeight="108">
    <path
        android:fillColor="#3DDC84"
        android:pathData="M0,0h108v108h-108z" />
</vector>
"##;
        self.add_file("app/src/main/res/drawable/ic_launcher_background.xml", background);

        // Generate launcher foreground (simple icon)
        let foreground = r##"<?xml version="1.0" encoding="utf-8"?>
<vector xmlns:android="http://schemas.android.com/apk/res/android"
    android:width="108dp"
    android:height="108dp"
    android:viewportWidth="108"
    android:viewportHeight="108">
    <group android:scaleX="0.92"
        android:scaleY="0.92"
        android:translateX="4.5"
        android:translateY="4.5">
        <path
            android:fillColor="#FFFFFF"
            android:pathData="M54,27C39.1,27 27,39.1 27,54s12.1,27 27,27s27,-12.1 27,-27S68.9,27 54,27zM54,72c-9.9,0 -18,-8.1 -18,-18s8.1,-18 18,-18s18,8.1 18,18S63.9,72 54,72z" />
    </group>
</vector>
"##;
        self.add_file("app/src/main/res/drawable/ic_launcher_foreground.xml", foreground);
    }

    // =========================================================================
    // Widget Files
    // =========================================================================

    /// Generate widget Kotlin files for all registered widgets
    fn generate_widget_files(&mut self) {
        let package = self.config.application_id.clone();
        let package_path = self.config.package_path();
        let widgets = self.config.widgets.clone();

        for widget_name in &widgets {
            let content = format!(
                r#"package {package}.ui.widgets
// Auto-generated by a2jet

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp

@Composable
fun {widget_name}(
    modifier: Modifier = Modifier
) {{
    var count by remember {{ mutableStateOf(0) }}

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {{
        Text(
            text = "Hello from Auto!",
            style = MaterialTheme.typography.headlineLarge,
            color = MaterialTheme.colorScheme.onBackground
        )

        Spacer(modifier = Modifier.height(16.dp))

        Button(onClick = {{ count++ }}) {{
            Text("Click Me")
        }}

        Text(
            text = "Count: $count",
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onBackground
        )
    }}
}}

@Preview(showBackground = true)
@Composable
fun {widget_name}Preview() {{
    {widget_name}()
}}
"#,
            );
            self.add_file(
                &format!("app/src/main/java/{}/ui/widgets/{}.kt", package_path, widget_name),
                &content,
            );
        }
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
            package = package,
            primary_80 = primary_80,
            secondary_80 = secondary_80,
            tertiary_80 = tertiary_80,
            primary_40 = primary_40,
            secondary_40 = secondary_40,
            tertiary_40 = tertiary_40,
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

    // =========================================================================
    // Plan 114: Hybrid Routing Support
    // =========================================================================

    /// Set the routes directory for convention-based route discovery
    pub fn with_routes_dir(mut self, routes_dir: std::path::PathBuf) -> Self {
        self.routes_dir = Some(routes_dir);
        self
    }

    /// Add routes from config (routes {} block)
    pub fn add_config_routes(&mut self, routes: Vec<crate::aura::AuraRoute>) {
        for route in routes {
            self.routes.push(crate::route::RouteDef::new(&route.path, &route.module)
                .with_source(RouteSource::Config));
        }
    }

    /// Discover routes from routes/ folder (convention-based)
    ///
    /// Returns the discovered routes or an empty vec if no routes directory
    pub fn discover_routes(&mut self) -> Vec<RouteDef> {
        if let Some(ref routes_dir) = &self.routes_dir {
            if routes_dir.exists() {
                let discovery = RouteDiscovery::new(routes_dir.clone());
                match discovery.discover() {
                    Ok(routes) => {
                        log::info!("Discovered {} routes from routes/ folder", routes.len());
                        return routes;
                    }
                    Err(e) => {
                        log::warn!("Failed to discover routes: {}", e);
                    }
                }
            }
        }
        Vec::new()
    }

    /// Merge discovered routes with config routes
    ///
    /// Config routes take precedence over convention routes when paths match
    pub fn merge_routes(&mut self) {
        let discovered = self.discover_routes();
        if discovered.is_empty() && self.routes.is_empty() {
            return;
        }

        // Extract config routes from current routes
        let config_routes: Vec<RouteDef> = self.routes.drain(..).collect();

        // Merge (config overrides convention)
        self.routes = RouteMerger::merge(discovered, config_routes);
    }

    /// Get merged routes
    pub fn get_routes(&self) -> &[RouteDef] {
        &self.routes
    }

    /// Check if there are any routes defined
    pub fn has_routes(&self) -> bool {
        !self.routes.is_empty()
    }

    /// Generate navigation screen from merged routes
    ///
    /// This creates a Navigation.kt file with NavHost containing all routes
    pub fn generate_navigation_file(&mut self) -> Option<String> {
        if self.routes.is_empty() {
            return None;
        }

        let package = &self.config.application_id;
        let mut nav_gen = super::NavigationGenerator::new();

        // Add all routes
        for route in &self.routes {
            // Convert path to screen name: /user/:id -> UserScreen
            let screen_name = path_to_screen_name(&route.path);
            nav_gen.add_route(&route.path, &screen_name);
        }

        // Generate NavHost
        let start_dest = self.routes.first()
            .map(|r| r.path.as_str())
            .unwrap_or("/");

        match nav_gen.generate_nav_host(start_dest) {
            Ok(code) => {
                let full_code = format!(
                    "package {package}\n\n{}\n\n{}",
                    nav_gen.generate_nav_imports(),
                    code
                );
                Some(full_code)
            }
            Err(_) => None,
        }
    }
}

/// Convert a route path to a screen name
///
/// # Examples
/// - `/` -> `HomeScreen`
/// - `/about` -> `AboutScreen`
/// - `/user/:id` -> `UserScreen`
/// - `/admin/settings` -> `AdminSettingsScreen`
fn path_to_screen_name(path: &str) -> String {
    let segments: Vec<&str> = path
        .split('/')
        .filter(|s| !s.is_empty() && !s.starts_with(':'))
        .collect();

    if segments.is_empty() {
        return "HomeScreen".to_string();
    }

    let name: String = segments
        .iter()
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect();

    format!("{}Screen", name)
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
        assert_eq!(config.compile_sdk, 35);
        assert_eq!(config.target_sdk, 35);
        // Default config has no widgets
        assert!(config.widgets.is_empty());
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
        assert!(content.contains("import com.test.app.ui.widgets.App"));
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
