export default class TestScript {
  speed = 5.0;
  name = "test";

  static executionOrder = 0;

  onStart(self) {
    console.log("TestScript started");
  }

  onUpdate(self, dt) {
    // Move logic would go here
  }

  onDestroy(self) {
    console.log("TestScript destroyed");
  }
}
