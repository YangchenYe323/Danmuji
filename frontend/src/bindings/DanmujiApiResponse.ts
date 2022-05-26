export interface DanmujiApiResponse<T> {
	success: boolean;
	payload: T | null;
}
