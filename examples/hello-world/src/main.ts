import { World, Transform, Sprite, input, Camera, MainCamera } from "longhorn";

export function onStart(world: World) {
  console.log("=== onStart called ===");

  // Create MainCamera entity (centered on the viewport)
  console.log("Spawning MainCamera at (640, 360)");
  world.spawn("MainCamera")
    .with(Camera, {
      position: { x: 640, y: 360 },  // Center of 1280x720 viewport
      zoom: 1.0
    })
    .with(MainCamera)
    .build();
  console.log("MainCamera spawned");

  // Create a simple entity with a sprite
  console.log("Spawning Player at (640, 360) with Sprite textureId=1, size=64x64");
  world.spawn("Player")
    .with(Transform, { x: 640, y: 360 })
    .with(Sprite, { textureId: 1, width: 64, height: 64 })
    .build();
  console.log("Player spawned");

  console.log("=== onStart complete ===");
}

export function onUpdate(world: World, dt: number) {
  const player = world.find("Player");
  if (!player) return;

  // Move player with touch
  if (input.justPressed()) {
    const pos = input.position();
    if (pos) {
      const transform = player.get(Transform);
      transform.x = pos.x;
      transform.y = pos.y;
    }
  }
}

export function onTouchStart(world: World, x: number, y: number) {
  console.log(`Touch at (${x}, ${y})`);
}
