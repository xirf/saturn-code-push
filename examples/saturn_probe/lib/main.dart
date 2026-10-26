import 'package:flutter/material.dart';
import 'package:shorebird_code_push/shorebird_code_push.dart';

const marker = String.fromEnvironment('PROBE_MARKER', defaultValue: 'baseline');

void main() => runApp(const ProbeApp());

class ProbeApp extends StatelessWidget {
  const ProbeApp({super.key});

  @override
  Widget build(BuildContext context) => MaterialApp(
    debugShowCheckedModeBanner: false,
    theme: ThemeData.dark(useMaterial3: true),
    home: const ProbeScreen(),
  );
}

class ProbeScreen extends StatefulWidget {
  const ProbeScreen({super.key});

  @override
  State<ProbeScreen> createState() => _ProbeScreenState();
}

class _ProbeScreenState extends State<ProbeScreen> {
  final updater = ShorebirdUpdater();
  String status = 'Reading updater…';
  bool busy = false;
  int current = 0;
  int next = 0;

  @override
  void initState() {
    super.initState();
    refresh();
  }

  Future<void> refresh() async {
    setState(() => busy = true);
    try {
      current = (await updater.readCurrentPatch())?.number ?? 0;
      next = (await updater.readNextPatch())?.number ?? 0;
      // These lines are the evidence consumed by the emulator script.
      // ignore: avoid_print
      print(
        'SATURN_PROBE marker=$marker current=$current next=$next available=${updater.isAvailable}',
      );
      if (!updater.isAvailable) {
        status = 'Build this app with the Shorebird engine to test updates.';
      } else {
        final update = await updater.checkForUpdate();
        status = update.name;
        // ignore: avoid_print
        print('SATURN_PROBE status=${update.name}');
        if (update == UpdateStatus.outdated) {
          await updater.update();
          next = (await updater.readNextPatch())?.number ?? 0;
          status = 'Patch $next downloaded. Restart the app.';
          // ignore: avoid_print
          print('SATURN_PROBE downloaded_next=$next');
        }
      }
    } catch (error) {
      status = error.toString();
      // ignore: avoid_print
      print('SATURN_PROBE error=$error');
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: const Text('Saturn · Android probe')),
    body: Padding(
      padding: const EdgeInsets.all(28),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const SizedBox(height: 40),
          Text(
            marker,
            style: const TextStyle(fontSize: 42, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 24),
          Text('Running patch: $current\nNext boot patch: $next'),
          const SizedBox(height: 24),
          Text(status),
          const SizedBox(height: 24),
          FilledButton(
            onPressed: busy ? null : refresh,
            child: const Text('Check and download'),
          ),
        ],
      ),
    ),
  );
}
