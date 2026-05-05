/*
 * Basic Arduino Test Sketch
 * Tests LED blinking and serial communication
 */
// Pin definitions
const int LED_PIN = 13;
const int ANALOG_PIN = A0;

// Timing variables
unsigned long previousMillis = 0;
const long interval = 1000;  // Blink interval in milliseconds

// State variables
bool ledState = false;
int readCount = 0;

void setup() {
  // Initialize serial communication
  Serial.begin(9600);

  // Initialize LED pin
  pinMode(LED_PIN, OUTPUT);
  pinMode(ANALOG_PIN, INPUT);

  // Print startup message
  Serial.println("=== Arduino Test Sketch ===");
  Serial.println("LED Pin: 13");
  Serial.println("Analog Pin: A0");
  Serial.println("Interval: 1000ms");
  Serial.println("==========================");
}

void loop() {
  unsigned long currentMillis = millis();

  // Blink LED without delay
  if (currentMillis - previousMillis >= interval) {
    previousMillis = currentMillis;

    // Toggle LED state
    ledState = !ledState;
    digitalWrite(LED_PIN, ledState);

    // Read analog value
    int analogValue = analogRead(ANALOG_PIN);

    // Print status
    readCount++;
    Serial.print("Count: ");
    Serial.print(readCount);
    Serial.print(" | LED: ");
    Serial.print(ledState ? "ON" : "OFF");
    Serial.print(" | Analog: ");
    Serial.println(analogValue);
  }
}

