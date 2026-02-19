import json
import re
from pathlib import Path

import pytest

from zopyx.pyjsonata import UNDEFINED, evaluate

ROOT = Path(__file__).resolve().parents[1]
TESTSUITE_DIRS = [ROOT / "tests" / "testsuite", ROOT / "tests" / "customsuite"]


def _iter_test_files():
    for base in TESTSUITE_DIRS:
        for path in base.rglob("*.json"):
            if "datasets" in path.parts:
                continue
            yield path


def _load_case_file(path: Path):
    data = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(data, list):
        return data
    return [data]


def _expr_from_case(case, path: Path) -> str:
    expr = case.get("expr")
    expr_file = case.get("expr-file")
    if isinstance(expr, str):
        return expr
    if isinstance(expr_file, str):
        expr_path = path.parent / expr_file
        return expr_path.read_text(encoding="utf-8")
    raise AssertionError(f"No expression found in {path}")


def _data_from_case(case, path: Path):
    dataset = case.get("dataset")
    if isinstance(dataset, str):
        suite = "customsuite" if "customsuite" in str(path) else "testsuite"
        dataset_path = ROOT / "tests" / suite / "datasets" / f"{dataset}.json"
        return json.loads(dataset_path.read_text(encoding="utf-8"))

    if "data" in case:
        return case["data"]

    return UNDEFINED


def _bindings_from_case(case):
    bindings = case.get("bindings")
    if isinstance(bindings, dict):
        return bindings
    return None


def _expected_error_code(case):
    if "error" in case and isinstance(case["error"], dict):
        return case["error"].get("code")
    return case.get("code")


def _extract_error_code(message: str):
    if not message:
        return None
    return message.split(" ", 1)[0]


def _build_cases():
    cases = []
    for path in sorted(_iter_test_files()):
        for idx, case in enumerate(_load_case_file(path)):
            case_id = f"{path.relative_to(ROOT)}::{idx}"
            cases.append((path, case, case_id))
    return cases


CASES = _build_cases()


@pytest.mark.parametrize("path,case,case_id", CASES, ids=[c[2] for c in CASES])
def test_testsuite_case(path, case, case_id):
    if "/skip/" in str(path):
        pytest.skip("Marked as skip in testsuite")

    expr = _expr_from_case(case, path)
    data = _data_from_case(case, path)
    bindings = _bindings_from_case(case)

    timelimit = case.get("timelimit")
    max_depth = case.get("depth")
    timelimit = timelimit if isinstance(timelimit, int) else None
    max_depth = max_depth if isinstance(max_depth, int) else None

    try:
        result = evaluate(expr, data, bindings, max_depth, timelimit)
    except ValueError as exc:
        expected_error = _expected_error_code(case)
        assert expected_error is not None, f"Unexpected error: {exc}"
        code = _extract_error_code(str(exc))
        assert code == expected_error
        return

    if case.get("undefinedResult") is True:
        assert result is UNDEFINED
        return

    if case.get("unordered") is True:
        expected = case.get("result", UNDEFINED)
        assert isinstance(expected, list)
        assert isinstance(result, list)
        for expected_item in expected:
            assert any(expected_item == actual for actual in result)
        return

    if isinstance(case.get("result_re"), str):
        assert isinstance(result, str)
        pattern = re.compile(case["result_re"])
        assert pattern.search(result) is not None
        return

    expected = case["result"] if "result" in case else UNDEFINED
    assert result == expected
