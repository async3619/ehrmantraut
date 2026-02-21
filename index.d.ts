/* eslint-disable */
import type { CstNode, IrModule } from './types'
export * from './types'

export declare function lower(source: string, language: string): IrModule
export declare function parse(source: string, language: string): CstNode
