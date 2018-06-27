// Type definitions for quick.db 6.3
// Project: https://github.com/TrueXPixels/quick.db#readme
// Definitions by: Nat Quayle Nelson <https://github.com/NQNStudios>
// Definitions: https://github.com/DefinitelyTyped/DefinitelyTyped

type OptionsWithTarget = {
    target?: string;
    table?: string;
};
type DataArray = Array<{ID: string, data: any}>;

declare module 'quick.db' {
// TODO is any the right type for data elements? They will be null if never set
// before, but can't be set to undefined (I think. Need to reproduce) and also
// can't be functions (I assume).

export function createWebview(password: string, port?: number): undefined;
export function set(ID: string, data: any, options?: OptionsWithTarget): Promise<any>;
export function fetch(ID: string, options?: OptionsWithTarget): Promise<any>;
export function fetchAll(options?: { table?: string }): Promise<DataArray>;
export function startsWith(text: string, options?: { sort?: string, table?: string }): Promise<DataArray>;
function _delete(ID: string, options?: { table?: string }): Promise<boolean>;
export { _delete as delete };
export function add(ID: string, value: number, options?: OptionsWithTarget): Promise<any>;
export function subtract(ID: string, value: number, options?: OptionsWithTarget): Promise<any>;
export function push(ID: string, data: any, options?: OptionsWithTarget): Promise<any>;

export class Table
{
    readonly name: string;
    fetch(Id: string, options?: OptionsWithTarget): Promise<any>;
    fetchAll(options?: { table?: string }): Promise<DataArray>;
    startsWith(text: string, options?: { sort?: string, table?: string }): Promise<DataArray>;
    delete(ID: string, options?: { table?: string }): Promise<boolean>;
    add(ID: string, value: number, options?: OptionsWithTarget): Promise<any>;
    subtract(ID: string, value: number, options?: OptionsWithTarget): Promise<any>;
    push(ID: string, data: any, options?: OptionsWithTarget): Promise<any>;
}
}
