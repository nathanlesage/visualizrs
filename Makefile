build: target/debug/bundle/osx/VisualizRS.app/Contents/Info.plist
  cargo bundle && ./scripts/customize_plist.sh

# TODO: Somehow add to the app bundle!
# <key>NSMicrophoneUsageDescription</key>
# <string>To visualize audio, this app needs access to the microphone.</string>
