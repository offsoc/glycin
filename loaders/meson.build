cargo_bin = find_program('cargo')

if get_option('profile') == 'release'
  rust_target = 'release'
  profile = 'release'
else
  rust_target = 'debug'
  profile = 'dev'
endif

target_dir = meson.project_build_root() / 'src'

cargo_options = [
  '--target-dir', target_dir,
  '--profile', profile,
]

cargo_env = {
  # Don't use $HOME/.cargo/ for this
  'CARGO_HOME': meson.project_build_root() / 'cargo',
  # Gettext translation
  'LOCALEDIR': get_option('prefix') / get_option('localedir'),
  # Git commit for debugging
  'GIT_DESCRIBE': run_command(
      'git',
      '-C', meson.project_source_root(),
      'rev-parse', '--short', 'HEAD',
      check: false
    ).stdout().strip(),
}

foreach loader : get_option('loaders')
  cargo_build = custom_target(
    loader,
    build_by_default: true,
    build_always_stale: true,
    output: loader + '-bin',
    console: true,
    env: cargo_env,
    command: [
      cargo_bin, 'build',
      [cargo_options, [ '--package', loader ]],
    ],
  )

  custom_target(
    loader + '-cp-binary',
    depends: cargo_build,
    build_by_default: true,
    build_always_stale: true,
    install: true,
    install_dir: libexecdir,
    output: loader,
    command: [
      'cp',
      target_dir / rust_target / loader,
      '@OUTPUT@',
    ],
  )

  i18n.merge_file(
      type: 'desktop',
      input: configure_file(
          input: loader / loader + '.conf',
          output: 'tmp-' + loader,
          configuration: { 'EXEC': libexecdir / loader }
      ),
      output: loader + '.conf',
      po_dir: '..' / 'po',
      install: true,
      install_dir: datadir / 'conf.d'
  )
endforeach