import numpy as np
from typing import Set

EPS = 1e-6
INF = float('inf')


def initialize_variables(A: np.ndarray, b: np.ndarray, c: np.ndarray):
    m, n = A.shape
    nA = np.concatenate((A, np.zeros((m, m))), axis=1)
    for i in range(m):
        nA[i, n + i] = 1
    nA = np.concatenate((np.zeros((n, n + m)), nA), axis=0)
    nc = np.concatenate((c, np.zeros(m)))
    nb = np.concatenate((np.zeros(n), b))
    N = set(range(n))
    B = set(range(n, n + m))
    v = 0
    return N, B, nA, nb, nc, v


def pivot(N: Set[int], B: Set[int], A: np.ndarray, b: np.ndarray, c: np.ndarray, v: float, l: int, e: int):
    # n = len(N) + len(B)
    nA = np.zeros(A.shape)
    nb = np.zeros(b.shape)
    nb[e] = b[l] / A[l, e]
    nA[e] = A[l, :] / A[l, e]
    for i in B - {l}:
        nb[i] = b[i] - A[i, e] * nb[e]
        nA[i] = A[i] - A[i, e] * nA[e]
    nv = v + c[e] * nb[e]
    nc = c - c[e] * nA[e]
    nN = (N - {e}) | {l}
    nB = (B - {l}) | {e}
    return nN, nB, nA, nb, nc, nv


def choose_index(c: np.ndarray) -> int:
    return np.argmax(c > 0)


def simplex_loop(N: Set[int], B: Set[int], A: np.ndarray, b: np.ndarray, c: np.ndarray, v: float):
    while max(c) > EPS:
        delta = np.full(c.shape, INF)
        e = choose_index(c)
        for i in B:
            if A[i, e] > -EPS:
                delta[i] = max(b[i] / A[i, e], 0)
            else:
                delta[i] = INF
        le = np.argmin(delta)
        if delta[le] == INF:
            return 'unbounded'
        else:
            N, B, A, b, c, v = pivot(N, B, A, b, c, v, le, e)
    return N, B, A, b, c, v


def initialize_simplex(A: np.ndarray, b: np.ndarray, c: np.ndarray):
    m, n = A.shape
    k = np.argmin(b)
    if b[k] > -EPS:
        return initialize_variables(A, b, c)
    pc = c

    A = np.concatenate((A, np.zeros((m, 1))), axis=1)
    i0 = n
    A[:, i0] = -1
    c = np.zeros(n + 1)
    c[i0] = -1
    N, B, A, b, c, v = initialize_variables(A, b, c)
    N, B, A, b, c, v = pivot(N, B, A, b, c, v, n + 1 + k, i0)
    sol = simplex_loop(N, B, A, b, c, v)
    if sol == 'unbounded':
        raise RuntimeError('Unexpected unbounded system when initializing simplex')
    N, B, A, b, c, v = sol
    if b[i0] == 0:
        if i0 in B:
            e = min(i for i in N if abs(A[0, i]) > EPS)
            N, B, A, b, c, v = pivot(N, B, A, b, c, v, i0, e)
        keep = np.ones(c.shape, np.bool)
        keep[i0] = False
        A = A[:, keep]
        b = b[keep]
        N = N - {i0}
        N = set(i for i in N if i < i0) | set(i - 1 for i in N if i > i0)
        B = set(i for i in B if i < i0) | set(i - 1 for i in B if i > i0)
        nc = np.zeros(n + m)
        v = 0
        for i in range(n):
            if i in B:
                nc -= A[i] * pc[i]
                nc[i] = 0
                v += b[i] * pc[i]
            else:
                nc[i] = pc[i]
        return N, B, A, b, c, v
    else:
        return 'infeasible'


def simplex(A: np.ndarray, b: np.ndarray, c: np.ndarray):
    n = A.shape[1]
    result = initialize_simplex(A, b, c)
    if type(result).__name__ == 'str':
        return result
    N, B, A, b, c, v = result
    result = simplex_loop(N, B, A, b, c, v)
    if type(result).__name__ == 'str':
        return result
    else:
        N, B, A, b, c, v = result
        return b[:n]


A = np.array([
    [2, -1],
    [1, -5],
])
b = np.array([2, -4])
c = np.array([2, -1])

x = simplex(A, b, c)
print(sum(c * x))
print(x)
