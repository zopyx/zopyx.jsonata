from ._native import Jsonata, UNDEFINED, UndefinedType as Undefined

__all__ = ["Jsonata", "UNDEFINED", "Undefined", "evaluate"]


def evaluate(expr, input=UNDEFINED, bindings=None, max_depth=None, time_limit=None):
    return Jsonata(expr).evaluate(input, bindings, max_depth, time_limit)
