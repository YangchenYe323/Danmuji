import type { GuardType } from "./GuardType";
import type { Medal } from "./Medal";

export interface DanmuMessage { uid: bigint, uname: string, content: string, is_gift_auto: boolean, sent_time: bigint, is_manager: boolean, is_vip: boolean, is_svip: boolean, is_full_member: boolean, medal: Medal | null, ul: bigint, ul_rank: string, guard: GuardType, }