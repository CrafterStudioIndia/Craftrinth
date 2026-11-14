cmake_minimum_required(VERSION 3.20)
project(CraftrinthWindows7 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Target Windows 7 (0x0601)
add_definitions(-D_WIN32_WINNT=0x0601 -DWINVER=0x0601 -DNTDDI_VERSION=0x06010000)
add_definitions(-DUNICODE -D_UNICODE)

# Memory Safety & Hardening Compiler Flags
if(MSVC)
	add_compile_options(
		/W4 /WX-
		/sdl
		/guard:cf
		/GS
		/Qspectre
	)
	add_link_options(
		/DYNAMICBASE
		/NXCOMPAT
		/HIGHENTROPYVA
		/guard:cf
	)
elseif(CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
	add_compile_options(
		-Wall -Wextra -Wpedantic
		-fstack-protector-strong
		-D_FORTIFY_SOURCE=2
		-Wformat -Wformat-security
		-fPIE
	)
	add_link_options(
		-Wl,--nxcompat
		-Wl,--dynamicbase
		-Wl,--high-entropy-va
		-pie
	)
endif()

include_directories(include)

file(GLOB_RECURSE SOURCES "src/*.cpp")

add_executable(craftrinth_win7 ${SOURCES})

# Windows Subsystem and Native Libraries
target_link_libraries(craftrinth_win7 PRIVATE
	opengl32
	gdi32
	user32
	shell32
	wininet
	ws2_32
	dxgi
)
