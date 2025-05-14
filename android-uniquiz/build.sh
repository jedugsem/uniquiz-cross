#!/bin/bash

xcargo ndk -t arm64-v8a -o app/src/main/jniLibs/  build --release
dxc ./gradlew clean
dxc ./gradlew build
adb install ./app/build/outputs/apk/debug/app-debug.apk

