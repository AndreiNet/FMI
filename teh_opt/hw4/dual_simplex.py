import numpy as np
from typing import Set, List

EPS = 1e-6


def find_base(A: np.ndarray, b: np.ndarray, c: np.ndarray) -> np.ndarray:
    return np.array(range(A.shape[1] - A.shape[0], A.shape[1]))


def simplex(A: np.ndarray, b: np.ndarray, c: np.ndarray):
    base = find_base(A, b, c)
    m, n = A.shape
    B = A[:, base]
    invB = np.linalg.inv(B)
    x = np.zeros(n)
    x[base] = np.matmul(invB, b)
    z = c * x
    y = np.zeros(A.shape)
    for j in range(n):
        y[:, j] = np.matmul(invB, A[:, j])
    while True:
        lv = np.argmin(x[base])
        le = base[lv]
        if x[le] >= -EPS:
            break
        if np.min(y[lv, :]) >= -EPS:
            return 'infeasible'
        inds = y[lv, :] < -EPS
        k = np.argmin(np.abs((z - c) / y[lv, :])[inds])
        k = np.array(range(n))[inds][k]

        nx = np.zeros(x.shape)
        nx[k] = x[le] / y[lv, k]
        for i, v in enumerate(list(base)):
            if i != lv:
                nx[v] = x[v] - y[i, k] * x[v] / y[lv, k]
        ny = np.zeros(y.shape)
        for j in range(n):
            ny[lv, j] = y[lv, j] / y[lv, k]
        for i in range(m):
            if i == lv:
                continue
            for j in range(n):
                ny[i, j] = y[i, j] - y[i, k] * x[i] / y[lv, k]
        nz = z - (z[k] - c[k]) * x[le] / y[lv, k]

        base[lv] = k
        B = A[:, base]
        x, y, z = nx, ny, nz
    return x


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


def pivot(N: Set[int], B: Set[int], A: np.ndarray, bl: List[np.ndarray], c: np.ndarray, vl: List[float],
          l: int, e: int):
    # n = len(N) + len(B)
    nA = np.zeros(A.shape)
    nA[e] = A[l, :] / A[l, e]
    for i in B - {l}:
        nA[i] = A[i] - A[i, e] * nA[e]
    nbl = []
    nvl = []
    for b, v in zip(bl, vl):
        nb = np.zeros(b.shape)
        nb[e] = b[l] / A[l, e]
        for i in B - {l}:
            nb[i] = b[i] - A[i, e] * nb[e]
        nbl.append(nb)
        nvl.append(v + c[e] * nb[e])
    nc = c - c[e] * nA[e]
    nN = (N - {e}) | {l}
    nB = (B - {l}) | {e}
    return nN, nB, nA, nbl, nc, nvl


def simplex_loop(N: Set[int], B: Set[int], A: np.ndarray, bl: List[np.ndarray], c: np.ndarray, vl: List[float]):
    m = A.shape[0]
    n = A.shape[1] - A.shape[0]
    while True:
        le = np.argmin(bl[0])
        if bl[0][le] >= -EPS:
            break
        inds = A[le, :] < 0
        if not np.any(inds):
            return 'infeasible'
        e = np.argmin((-c / A[le, :])[inds])
        e = np.array(range(n + m))[inds][e]
        N, B, A, bl, c, vl = pivot(N, B, A, bl, c, vl, le, e)
    return N, B, A, bl, c, vl


def initialize_simplex(A: np.ndarray, b: np.ndarray, c: np.ndarray):
    m, n = A.shape
    k = np.argmax(c)
    if c[k] < EPS:
        return initialize_variables(A, b, c)
    pb = np.concatenate((np.zeros(n), b, [0]))
    pv = 0
    A = np.concatenate((A, np.ones((1, n))))
    b = np.zeros(m + 1)
    b[m] = 1
    N, B, A, b, c, v = initialize_variables(A, b, c)
    i0 = n + m
    N, B, A, [b, pb], c, [v, pv] = pivot(N, B, A, [b, pb], c, [v, pv], i0, k)
    sol = simplex_loop(N, B, A, [b, pb], c, [v, pv])
    if sol == 'infeasible':
        raise RuntimeError('Unexpected unbounded system when initializing simplex')
    N, B, A, [b, pb], c, [v, pv] = sol
    if v != 0:
        return 'unbounded'
    if i0 in N:
        k = min(i for i in B if abs(A[i, i0]) > EPS)
        N, B, A, [b, pb], c, [v, pv] = pivot(N, B, A, [b, pb], c, [v, pv], k, i0)
    keep = np.ones(n + m + 1, np.bool)
    keep[n + m] = False
    A = A[keep, :]
    A = A[:, keep]
    B = B - {n + m}
    b = pb[keep]
    c = c[keep]
    v = pv
    return N, B, A, b, c, v


def simplex2(A: np.ndarray, b: np.ndarray, c: np.ndarray):
    m, n = A.shape
    result = initialize_simplex(A, b, c)
    if type(result).__name__ == 'str':
        return result
    N, B, A, b, c, v = result
    result = simplex_loop(N, B, A, [b], c, [v])
    if type(result).__name__ == 'str':
        return result
    N, B, A, [b], c, [v] = result
    return b[:n]


A = np.array([
    [2, -1],
    [1, -5],
])

b = np.array([2, -4])
c = np.array([2, -1])

x = simplex2(A, b, c)
print(sum(c * x))
print(x)
