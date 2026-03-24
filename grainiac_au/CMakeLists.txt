cmake_minimum_required(VERSION 3.15)
project(OhMightyGrain) # <-- change me

# this CMakeLists.txt has a "gain_auv2" target that
# converts the "gain.clap" file into AUv2 plugin "gain.component".

# assumes the "gain.clap" file is located next to this CMakeLists.txt
set(CLAP_PLUGIN_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../grainiac_plugin_gui/target/bundled/grainiac_plugin_gui.clap") # <-- change me

# the following lines are required to build AU SDK on macOS
set(CMAKE_CXX_STANDARD 17)
enable_language(OBJC)
enable_language(OBJCXX)

# ensure clap-wrapper lives as a submodule next to this CMakeLists.txt
# https://github.com/free-audio/clap-wrapper/
set(CLAP_WRAPPER_DOWNLOAD_DEPENDENCIES ON)
add_subdirectory(clap-wrapper)

# create AUv2 target
set(AUV2_TARGET ${PROJECT_NAME}_auv2)
add_library(${AUV2_TARGET} MODULE)

# use clap-wrapper to populate the AUv2 target
target_add_auv2_wrapper(
        TARGET ${AUV2_TARGET}
        MACOS_EMBEDDED_CLAP_LOCATION ${CLAP_PLUGIN_PATH}
        
        # change all of the below
        OUTPUT_NAME "Grainiac"
        BUNDLE_IDENTIFIER "com.timerift.grainiac"
        BUNDLE_VERSION "1.0"
        MANUFACTURER_NAME "Timerift Audio "
        MANUFACTURER_CODE "Tmrf"
        SUBTYPE_CODE "Grnc"
        INSTRUMENT_TYPE "aumu"
)
