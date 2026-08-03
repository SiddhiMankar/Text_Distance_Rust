import textdistance

mat = {("cat", "bat"): 0.5}
m = textdistance.Matrix(mat=mat)

print(m.similarity("cat", "bat"))              # exact match in mat
print(m.similarity("bat", "cat"))              # reversed — tests symmetric fallback
print(m.similarity("dog", "dog"))              # not in mat, but identical — identity fallback
print(m.similarity("dog", "fox"))              # not in mat, not identical — mismatch_cost fallback
print(m.maximum("cat", "bat"))                 # should always be match_cost, regardless of input
print(m.distance("cat", "bat"))
print(m.normalized_similarity("cat", "bat"))