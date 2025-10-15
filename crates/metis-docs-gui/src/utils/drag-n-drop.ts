export function applyDrag(arr: any[], dragResult: any) {
  const { addedIndex, payload, removedIndex } = dragResult;
  if (removedIndex === null && addedIndex === null) return arr;

  const result = [...arr];
  let itemToAdd = payload;

  let actualRemovedIndex = removedIndex;
  if (removedIndex !== null) {
    // Find the index of the payload in the current array
    actualRemovedIndex = result.findIndex(item => 
      item === payload || (item.id && payload.id && item.id === payload.id)
    );
    if (actualRemovedIndex !== -1) {
      itemToAdd = result.splice(actualRemovedIndex, 1)[0];
    }
  }

  if (addedIndex !== null) {
    result.splice(addedIndex, 0, itemToAdd);
  }

  return result;
}