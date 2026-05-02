import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

export interface Connection {
  id: string;
  name: string;
  driver: string;
  host: string;
}

export const useConnectionStore = defineStore('connection', {
  state: () => ({
    connections: [] as Connection[],
    activeConnectionId: null as string | null,
    loading: false,
  }),

  actions: {
    async fetchConnections() {
      this.loading = true;
      try {
        this.connections = await invoke<Connection[]>('cmd_get_connections');
        this.activeConnectionId = await invoke<string | null>('cmd_get_active_connection_id');
      } catch (error) {
        console.error('获取连接列表失败:', error);
      } finally {
        this.loading = false;
      }
    },

    async addConnection(name: string, driver: string, host: string) {
      try {
        await invoke('cmd_add_connection', { name, driver, host });
        await this.fetchConnections();
      } catch (error) {
        console.error('添加连接失败:', error);
        throw error;
      }
    },

    async deleteConnection(id: string) {
      try {
        await invoke('cmd_delete_connection', { id });
        await this.fetchConnections();
      } catch (error) {
        console.error('删除连接失败:', error);
        throw error;
      }
    },

    async switchConnection(id: string) {
      this.loading = true;
      try {
        await invoke('cmd_switch_connection', { id });
        this.activeConnectionId = id;
        // 切换连接后可能需要重新加载容器等数据
      } catch (error) {
        console.error('切换连接失败:', error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
  },
});
