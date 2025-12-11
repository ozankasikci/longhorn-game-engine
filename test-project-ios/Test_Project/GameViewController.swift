import UIKit
import MetalKit

class GameViewController: UIViewController {
    var metalView: MTKView!
    var displayLink: CADisplayLink?

    override func viewDidLoad() {
        super.viewDidLoad()

        print("=== SWIFT: viewDidLoad called")

        // Initialize logging
        print("=== SWIFT: Calling longhorn_init()")
        guard longhorn_init() else {
            fatalError("Failed to initialize Longhorn")
        }
        print("=== SWIFT: longhorn_init() returned true")

        // Set up Metal view FIRST
        metalView = MTKView(frame: view.bounds)
        metalView.device = MTLCreateSystemDefaultDevice()
        metalView.clearColor = MTLClearColor(red: 0.1, green: 0.1, blue: 0.1, alpha: 1.0) // Dark gray
        metalView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        view.addSubview(metalView)

        // Get the Metal layer
        guard let metalLayer = metalView.layer as? CAMetalLayer else {
            fatalError("Failed to get CAMetalLayer from MTKView")
        }
        let layerPointer = Unmanaged.passUnretained(metalLayer).toOpaque()

        // Create engine with Metal renderer
        let width = UInt32(metalView.bounds.width)
        let height = UInt32(metalView.bounds.height)

        print("=== SWIFT: Calling longhorn_create_with_metal with layer: \(layerPointer), size: \(width)x\(height)")
        guard longhorn_create_with_metal(layerPointer, width, height) else {
            fatalError("Failed to create engine with Metal")
        }
        print("=== SWIFT: longhorn_create_with_metal() returned true")

        // Load game AFTER renderer is ready
        if let gamePath = Bundle.main.resourcePath?.appending("/GameResources") {
            print("=== SWIFT: Calling longhorn_load_game with path: \(gamePath)")
            let cPath = (gamePath as NSString).utf8String
            guard longhorn_load_game(cPath) else {
                fatalError("Failed to load game from: \(gamePath)")
            }
            print("=== SWIFT: longhorn_load_game() returned true")
        }

        // Start game
        print("=== SWIFT: Calling longhorn_start()")
        guard longhorn_start() else {
            fatalError("Failed to start game")
        }
        print("=== SWIFT: longhorn_start() returned true")

        // Start game loop
        displayLink = CADisplayLink(target: self, selector: #selector(gameLoop))
        displayLink?.add(to: .main, forMode: .default)
    }

    @objc func gameLoop() {
        let _ = longhorn_update()
    }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        if let touch = touches.first {
            let location = touch.location(in: view)
            longhorn_handle_touch_start(Float(location.x), Float(location.y))
        }
    }

    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        if let touch = touches.first {
            let location = touch.location(in: view)
            longhorn_handle_touch_move(Float(location.x), Float(location.y))
        }
    }

    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        if let touch = touches.first {
            let location = touch.location(in: view)
            longhorn_handle_touch_end(Float(location.x), Float(location.y))
        }
    }

    override func viewWillTransition(to size: CGSize, with coordinator: UIViewControllerTransitionCoordinator) {
        super.viewWillTransition(to: size, with: coordinator)
        longhorn_resize(UInt32(size.width), UInt32(size.height))
    }

    deinit {
        displayLink?.invalidate()
        longhorn_cleanup()
    }
}
