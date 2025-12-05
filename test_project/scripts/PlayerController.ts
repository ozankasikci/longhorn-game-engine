// PlayerController.ts - Test script for Longhorn engine
export default class PlayerController {
  // Properties editable in inspector
  speed = 5.0;
  jumpHeight = 2.0;
  playerName = "Hero";

  // Lower numbers run first
  static executionOrder = 0;

  onStart(self: any) {
    console.log(`[PlayerController] ${this.playerName} spawned with speed=${this.speed}`);
  }

  onUpdate(self: any, dt: number) {
    // This will move the entity once component access ops are fully wired
    console.log(`[PlayerController] Update: dt=${dt.toFixed(3)}, speed=${this.speed}`);
  }

  onDestroy(self: any) {
    console.log(`[PlayerController] ${this.playerName} destroyed`);
  }
}
