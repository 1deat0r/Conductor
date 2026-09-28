# Android and iPhone shell

Run `pnpm dev:mobile` from the root for Metro, then an appropriate native development client. `pnpm --filter @conductor/mobile android` needs Android SDK/JDK/emulator or a device. `pnpm --filter @conductor/mobile ios` needs macOS/Xcode. Local native generation creates ignored android/ and ios/ directories from the pinned Expo configuration.

The identifiers dev.conductor.scaffold are local placeholders; no EAS project, store account, provisioning profile, push service, credentials or remote agent is configured. Before any store build choose owned identifiers and pass the mobile qualification gate. This source supports both platforms but typechecking/export is not a native build or physical-device test. S0 contains no WebView, push integration, cache encryption or approval key handling; those require the SPEC gates before activation.
