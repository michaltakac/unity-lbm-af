using System;
using System.Runtime.InteropServices;
using System.Text;
using UnityEngine;

public class LBM
{
    private static IntPtr _sim_handle;
	private static IntPtr _data_handle;
	private UInt32 sim_timestep = 0;

	[DllImport("lbmaf")]
    private static extern bool init_sim(out IntPtr sim_handle, out IntPtr data_handle);
	public static bool InitSimulation()
    {
        return init_sim(out _sim_handle, out _data_handle);
    }

	[DllImport("lbmaf")]
    private static extern void dispose(out IntPtr sim_handle);
	public static void DisposeSim()
    {
        dispose(out _sim_handle);
    }

	[DllImport("lbmaf")]
    private static extern void simulate(IntPtr handle);
	public static void SimulateNextIteration()
    {
        simulate(_sim_handle);
    }

	[DllImport("lbmaf")]
    private static extern void get_sim_data(IntPtr handle);
	public static void GetSimData()
    {
        get_sim_data(_sim_handle);
    }

    public void CopyResultsToBuffer(byte[] buffer, UInt32 size) {
        Marshal.Copy((IntPtr)_data_handle, buffer, 0, size);
    }
}