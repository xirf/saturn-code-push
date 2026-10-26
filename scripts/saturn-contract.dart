// Run with the pinned Shorebird workspace's --packages configuration.
// This checks its actual Dart response decoders against the live Saturn API.
import 'dart:io';
import 'package:crypto/crypto.dart';
import 'package:shorebird_code_push_client/shorebird_code_push_client.dart';

Future<void> main(List<String> args) async {
  final root = Directory.current.path;
  final token = File('$root/server/.env').readAsLinesSync()
      .firstWhere((line) => line.startsWith('ADMIN_TOKEN='))
      .substring('ADMIN_TOKEN='.length);
  final client = CodePushClient(
    hostedUri: Uri.parse(args.isEmpty ? 'http://127.0.0.1:3000' : args.first),
    customHeaders: {'Authorization': 'Bearer $token'},
  );
  final user = await client.getCurrentUser();
  if (user == null) throw StateError('Missing user');
  final orgs = await client.getOrganizationMemberships();
  if (orgs.isEmpty) throw StateError('Missing organization');
  final app = await client.createApp(organizationId: orgs.first.organization.id, displayName: 'Protocol smoke ${DateTime.now().millisecondsSinceEpoch}');
  final channels = await client.getChannels(appId: app.id);
  await client.getApp(appId: app.id);
  await client.getApps();
  final release = await client.createRelease(appId: app.id, version: '1.0.0+1', flutterRevision: 'protocol-smoke');
  final file = File('$root/.tooling/saturn-contract.bin')..writeAsBytesSync([1,2,3]);
  final hash = sha256.convert(file.readAsBytesSync()).toString();
  await client.createReleaseArtifact(artifactPath: file.path, appId: app.id, releaseId: release.id, arch: 'x86_64', platform: ReleasePlatform.android, hash: hash, canSideload: false, podfileLockHash: null);
  await client.updateReleaseStatus(appId: app.id, releaseId: release.id, platform: ReleasePlatform.android, status: ReleaseStatus.active);
  await client.getReleaseArtifacts(appId: app.id, releaseId: release.id);
  await client.getReleases(appId: app.id);
  final patch = await client.createPatch(appId: app.id, releaseId: release.id, metadata: {});
  await client.createPatchArtifact(artifactPath: file.path, appId: app.id, patchId: patch.id, arch: 'x86_64', platform: ReleasePlatform.android, hash: hash);
  await client.promotePatch(appId: app.id, patchId: patch.id, channelId: channels.first.id);
  final patches = await client.getPatches(appId: app.id, releaseId: release.id);
  if (patches.length != 1) throw StateError('Patch response mismatch');
  await client.rollbackPatch(appId: app.id, releaseId: release.id, patchId: patch.id);
  await client.rollforwardPatch(appId: app.id, releaseId: release.id, patchId: patch.id);
  print('PASS: actual Shorebird Dart client decoded management responses and completed both multipart upload flows.');
  print('Protocol fixture app: ${app.id}. These fixture bytes are not an executable patch.');
  exit(0);
}
