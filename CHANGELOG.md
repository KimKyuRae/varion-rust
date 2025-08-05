# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.3] - 2025-08-05

### Added
- `@next` 지시어 추가: 선택지 없이 다음 노드를 지정하는 기능

### Changed
- 파서가 `@next` 지시어를 처리하도록 업데이트
- `@next`와 선택지가 동시에 사용될 경우 에러를 발생시키도록 파서 로직 수정

## [0.0.2] - 2025-08-04

### Added
- 노드에 태그를 추가할 수 있는 기능 (`#tag` 형식)
- 주석 (`//`) 파싱 기능 추가
- 새로운 예제 파일 (`examples/varion_examples.va` 내용 변경)

### Changed
- 프로젝트 버전이 `0.0.1`에서 `0.0.2`로 업데이트 (`Cargo.toml`, `Cargo.lock`)
- 기존 테스트 코드의 문자열 포맷팅 변경

### Fixed

## [0.0.1]

- Initial release.
