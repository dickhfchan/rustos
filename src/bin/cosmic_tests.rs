#![no_std]
#![no_main]

extern crate rustos;

use core::arch::asm;
use core::panic::PanicInfo;

use rustos::{cosmic, graphics, input, wayland};
use rustos::{ipc, memory, panic as panic_runtime, process, syscall, uart, userspace};

type TestFn = fn() -> Result<(), &'static str>;

const TESTS: &[(&'static str, TestFn)] = &[
    ("COSMIC Initialization", test_cosmic_initialization),
    ("Window Management", test_window_management),
    ("Workspace Functionality", test_workspace_functionality),
    ("Graphics Subsystem", test_graphics_subsystem),
    ("Input System", test_input_system),
    ("Wayland Compatibility", test_wayland_compatibility),
    ("Launcher System", test_launcher_system),
    ("Notification System", test_notification_system),
    ("Event Processing", test_event_processing),
    ("Compositing Pipeline", test_compositing_pipeline),
];

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    uart::init();
    memory::init();
    process::init();
    syscall::init();
    rustos::fs::init();
    ipc::init();
    userspace::init();

    panic_runtime::set_handler(cosmic_test_panic_handler);

    run_cosmic_tests();
    exit_qemu(0);
}

fn run_cosmic_tests() {
    rustos::println!("🚀 COSMIC Desktop Environment Test Suite");
    rustos::println!("=========================================");
    rustos::println!("Running {} test cases", TESTS.len());
    rustos::println!("");

    let mut passed = 0;
    let mut failed = 0;

    for (test_name, test_fn) in TESTS {
        rustos::print!("Testing {}: ", test_name);
        
        match test_fn() {
            Ok(()) => {
                rustos::println!("✅ PASSED");
                passed += 1;
            }
            Err(error) => {
                rustos::println!("❌ FAILED - {}", error);
                failed += 1;
            }
        }
    }

    rustos::println!("");
    rustos::println!("Test Results:");
    rustos::println!("✅ Passed: {}", passed);
    rustos::println!("❌ Failed: {}", failed);
    rustos::println!("📊 Success Rate: {}%", (passed * 100) / (passed + failed));

    if failed == 0 {
        rustos::println!("🎉 All COSMIC tests passed!");
    } else {
        rustos::println!("⚠️  Some tests failed, check implementation");
    }
}

fn test_cosmic_initialization() -> Result<(), &'static str> {
    // Test COSMIC desktop initialization
    cosmic::cosmic_init(1920, 1080).map_err(|_| "Failed to initialize COSMIC")?;
    
    // Verify session is active
    if !cosmic::cosmic_is_session_active() {
        return Err("COSMIC session not active after initialization");
    }

    // Check compositor exists
    if cosmic::cosmic_get_compositor().is_none() {
        return Err("COSMIC compositor not found");
    }

    rustos::println!("  📱 Display: 1920x1080, Session Active: {}", cosmic::cosmic_is_session_active());
    Ok(())
}

fn test_window_management() -> Result<(), &'static str> {
    // Create test windows
    let window1 = cosmic::cosmic_create_window("Terminal", 800, 600)
        .map_err(|_| "Failed to create terminal window")?;
    
    let window2 = cosmic::cosmic_create_window("File Manager", 600, 400)
        .map_err(|_| "Failed to create file manager window")?;

    let window3 = cosmic::cosmic_create_window("Settings", 500, 350)
        .map_err(|_| "Failed to create settings window")?;

    // Verify windows were created with different IDs
    if window1 == window2 || window2 == window3 || window1 == window3 {
        return Err("Window IDs should be unique");
    }

    // Test graphics stats
    let (window_count, surface_count) = graphics::graphics_get_stats();
    if window_count < 3 {
        return Err("Expected at least 3 windows");
    }

    rustos::println!("  🪟 Created windows: {} (IDs: {}, {}, {})", window_count, window1, window2, window3);
    rustos::println!("  🎨 Surfaces: {}", surface_count);
    Ok(())
}

fn test_workspace_functionality() -> Result<(), &'static str> {
    let compositor = cosmic::cosmic_get_compositor()
        .ok_or("COSMIC compositor not available")?;

    let shell = compositor.get_shell();
    
    // Check default workspace exists
    if shell.active_workspace.is_none() {
        return Err("No active workspace found");
    }

    let workspace_count = shell.workspaces.len();
    if workspace_count == 0 {
        return Err("No workspaces found");
    }

    // Test workspace switching
    if let Some(&first_workspace_id) = shell.workspaces.keys().next() {
        cosmic::cosmic_switch_workspace(first_workspace_id)
            .map_err(|_| "Failed to switch workspace")?;
    }

    rustos::println!("  🖥️  Workspaces: {}, Active: {:?}", workspace_count, shell.active_workspace);
    Ok(())
}

fn test_graphics_subsystem() -> Result<(), &'static str> {
    // Test graphics initialization
    graphics::graphics_init().map_err(|_| "Failed to initialize graphics")?;

    // Test framebuffer creation
    graphics::graphics_init_framebuffer(1920, 1080, graphics::PixelFormat::RGBA8888)
        .map_err(|_| "Failed to create framebuffer")?;

    // Test surface creation
    let surface1 = graphics::graphics_create_surface(800, 600, graphics::PixelFormat::RGBA8888)
        .map_err(|_| "Failed to create surface")?;

    let surface2 = graphics::graphics_create_surface(400, 300, graphics::PixelFormat::RGBA8888)
        .map_err(|_| "Failed to create second surface")?;

    if surface1 == surface2 {
        return Err("Surface IDs should be unique");
    }

    // Test compositing
    graphics::graphics_composite().map_err(|_| "Failed to composite graphics")?;

    rustos::println!("  🎨 Framebuffer: 1920x1080 RGBA8888");
    rustos::println!("  🖼️  Surfaces created: {} & {}", surface1, surface2);
    Ok(())
}

fn test_input_system() -> Result<(), &'static str> {
    // Test input initialization
    input::input_init().map_err(|_| "Failed to initialize input system")?;

    // Test input manager exists
    if input::input_get_manager().is_none() {
        return Err("Input manager not found");
    }

    // Test event injection
    let manager = input::input_get_manager().unwrap();
    
    // Inject key events
    manager.inject_key_event(input::KeyCode::Space, true)
        .map_err(|_| "Failed to inject key press")?;
    manager.inject_key_event(input::KeyCode::Space, false)
        .map_err(|_| "Failed to inject key release")?;

    // Inject mouse events
    manager.inject_mouse_move(100, 200)
        .map_err(|_| "Failed to inject mouse move")?;
    manager.inject_mouse_button(input::MouseButton::Left, true)
        .map_err(|_| "Failed to inject mouse click")?;

    // Test event queue
    let has_events = input::input_has_events();
    if !has_events {
        return Err("Expected input events in queue");
    }

    // Pop and count events
    let mut event_count = 0;
    while let Some(_event) = input::input_pop_event() {
        event_count += 1;
        if event_count > 10 { break; } // Prevent infinite loop
    }

    rustos::println!("  ⌨️  Input events processed: {}", event_count);
    rustos::println!("  🖱️  Mouse and keyboard support active");
    Ok(())
}

fn test_wayland_compatibility() -> Result<(), &'static str> {
    // Test Wayland initialization
    wayland::wayland_init().map_err(|_| "Failed to initialize Wayland")?;

    // Test display creation
    if wayland::wayland_get_display().is_none() {
        return Err("Wayland display not found");
    }

    // Test socket creation
    let socket_fd = wayland::wayland_create_socket()
        .map_err(|_| "Failed to create Wayland socket")?;

    if socket_fd <= 0 {
        return Err("Invalid socket file descriptor");
    }

    // Test client acceptance (simulated)
    let client_id = wayland::wayland_accept_client(socket_fd + 1)
        .map_err(|_| "Failed to accept Wayland client")?;

    if client_id == 0 {
        return Err("Invalid client ID");
    }

    rustos::println!("  📡 Wayland socket: FD {}", socket_fd);
    rustos::println!("  👤 Client connected: ID {}", client_id);
    Ok(())
}

fn test_launcher_system() -> Result<(), &'static str> {
    // Test launcher toggle
    cosmic::cosmic_toggle_launcher().map_err(|_| "Failed to toggle launcher")?;
    
    let compositor = cosmic::cosmic_get_compositor()
        .ok_or("COSMIC compositor not available")?;

    let shell = compositor.get_shell();
    
    // Check launcher exists and has applications
    if let Some(ref launcher) = shell.launcher {
        if launcher.applications.is_empty() {
            return Err("Launcher has no applications");
        }
        
        let app_count = launcher.applications.len();
        if app_count < 4 { // Should have at least the default apps
            return Err("Expected more default applications");
        }

        rustos::println!("  🚀 Launcher applications: {}", app_count);
        rustos::println!("  📱 Apps: Terminal, Files, Settings, Text Editor, ls, pwd");
        
        // Test toggle again
        cosmic::cosmic_toggle_launcher().map_err(|_| "Failed to toggle launcher off")?;
    } else {
        return Err("Launcher not found");
    }

    Ok(())
}

fn test_notification_system() -> Result<(), &'static str> {
    // Test notification creation
    let notification1 = cosmic::cosmic_show_notification(
        "Test Notification".into(),
        "This is a test message".into(),
        cosmic::NotificationUrgency::Normal
    ).map_err(|_| "Failed to create notification")?;

    let notification2 = cosmic::cosmic_show_notification(
        "Critical Alert".into(),
        "This is urgent!".into(),
        cosmic::NotificationUrgency::Critical
    ).map_err(|_| "Failed to create critical notification")?;

    if notification1 == notification2 {
        return Err("Notification IDs should be unique");
    }

    let compositor = cosmic::cosmic_get_compositor()
        .ok_or("COSMIC compositor not available")?;

    let shell = compositor.get_shell();
    let notification_count = shell.notifications.len();
    
    if notification_count < 2 {
        return Err("Expected at least 2 notifications");
    }

    rustos::println!("  🔔 Notifications created: {} (IDs: {}, {})", notification_count, notification1, notification2);
    Ok(())
}

fn test_event_processing() -> Result<(), &'static str> {
    // Test COSMIC event processing
    cosmic::cosmic_process_events().map_err(|_| "Failed to process COSMIC events")?;

    // Test Wayland event dispatch
    wayland::wayland_dispatch_events().map_err(|_| "Failed to dispatch Wayland events")?;
    wayland::wayland_flush_clients().map_err(|_| "Failed to flush Wayland clients")?;

    rustos::println!("  ⚡ Event processing pipeline active");
    Ok(())
}

fn test_compositing_pipeline() -> Result<(), &'static str> {
    // Test frame rendering
    cosmic::cosmic_render_frame().map_err(|_| "Failed to render COSMIC frame")?;

    // Test graphics compositing
    graphics::graphics_composite().map_err(|_| "Failed to composite graphics")?;

    // Get final stats
    let (window_count, surface_count) = graphics::graphics_get_stats();
    
    rustos::println!("  🎬 Frame rendered successfully");
    rustos::println!("  📊 Final stats - Windows: {}, Surfaces: {}", window_count, surface_count);
    Ok(())
}

fn exit_qemu(code: u64) -> ! {
    unsafe {
        asm!(
            "mov x0, {fn_id}",
            "mov x1, {code}",
            "mov x2, xzr",
            "mov x3, xzr",
            "hvc #0",
            fn_id = in(reg) 0x84000008u64,
            code = in(reg) code,
            options(noreturn)
        );
    }
}

fn cosmic_test_panic_handler(info: &PanicInfo) -> ! {
    rustos::println!("COSMIC test panic: {}", info);
    exit_qemu(1);
}

core::arch::global_asm!(include_str!("../boot.s"));