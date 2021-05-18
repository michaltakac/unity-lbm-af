using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using UnityEngine;

public class Sim : MonoBehaviour
{
	private static byte[] buffer;
	public UInt32 size;
	public UInt32 domain_width = 128;
	public UInt32 domain_height = 128;
	private Texture2D image;
	

	void Start()
	{
		size = domain_width * domain_height * 4;
		buffer = new byte[size];
		image = new Texture2D(domain_width, domain_height, TextureFormat.RGBA32, false);
		GetComponent<Renderer>().material.mainTexture = image;

        bool isSimReady = InitSimulation();
		if (!isSimReady) return;

		Debug.Log("Simulation initialized.");
		StartCoroutine("GetData");
	}


	public IEnumerator GetData()
    {
		while(true) {
			LBM.SimulateNextIteration();
			LBM.GetSimData();
			LBM.CopyResultsToBuffer((IntPtr)_data_handle, buffer, size);
			image.LoadRawTextureData(buffer);
			image.Apply();
			sim_timestep++;
			yield return null;
		}
    }
	
	void Update()
	{

	}

	void OnDestroy () {
		// DisposeSim(); // TODO: implement disposing on Rust side
	}
}
