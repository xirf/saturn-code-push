import 'package:flutter/material.dart';
import 'package:shorebird_code_push/shorebird_code_push.dart';

void main() => runApp(const MaterialApp(home: RolloutExample()));

class RolloutExample extends StatefulWidget {
  const RolloutExample({super.key});

  @override
  State<RolloutExample> createState() => _RolloutExampleState();
}

class _RolloutExampleState extends State<RolloutExample> {
  final _updater = ShorebirdUpdater();
  // Use the same track for both check and download.
  static const _track = UpdateTrack.stable;
  bool _busy = false;
  String _message = 'Ready to check for updates.';

  Future<void> _check() async {
    if (_busy) return;
    setState(() => _busy = true);
    var message = '';
    try {
      if (!_updater.isAvailable) {
        message = 'Build with Shorebird to enable updates.';
      } else {
        // The native updater persists and sends client_id automatically.
        // Do not generate another ID in Dart or make a separate eligibility check.
        final status = await _updater.checkForUpdate(track: _track);
        switch (status) {
          case UpdateStatus.outdated:
            await _updater.update(track: _track);
            message = 'Update downloaded. Close and reopen the app to apply it.';
          case UpdateStatus.restartRequired:
            message = 'Close and reopen the app to finish applying update changes.';
          case UpdateStatus.upToDate:
            message = 'No update is available for this installation right now.';
          case UpdateStatus.unavailable:
            message = 'Updates are unavailable in this build.';
        }
      }
    } on UpdateException catch (error) {
      // Rollout settings may change between the check and download.
      message = error.reason == UpdateFailureReason.noUpdate
          ? 'No update is available for this installation right now.'
          : 'Update failed. You can retry later.';
    } catch (_) {
      message = 'Could not check for updates. You can retry later.';
    } finally {
      if (mounted) {
        setState(() {
          _busy = false;
          _message = message;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: const Text('Saturn rollout example')),
    body: Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(_message),
          const SizedBox(height: 16),
          FilledButton(
            onPressed: _busy ? null : _check,
            child: Text(_busy ? 'Checking…' : 'Check for updates'),
          ),
        ],
      ),
    ),
  );
}
